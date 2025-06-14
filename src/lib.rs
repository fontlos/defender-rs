mod com;
mod ctx;
mod ipc;
pub mod loader;
mod utils;

use windows::Win32::{
    Foundation::{CloseHandle, HINSTANCE},
    System::{
        Com::CoInitialize,
        Threading::{CreateThread, ExitProcess, THREAD_CREATION_FLAGS},
    },
};
use windows::core::BSTR;

use std::ffi::c_void;
use std::sync::Once;

static START: Once = Once::new();

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_hinst: HINSTANCE, reason: u32, _reserved: *mut c_void) -> i32 {
    const DLL_PROCESS_ATTACH: u32 = 1;
    if reason == DLL_PROCESS_ATTACH {
        START.call_once(|| {
            let h = unsafe {
                CreateThread(
                    None,
                    0,
                    Some(entry_thread),
                    None, // 传递 hinst
                    THREAD_CREATION_FLAGS::default(),
                    None,
                )
            };
            if let Ok(handle) = h {
                let _ = unsafe { CloseHandle(handle) };
            }
        });
    }
    1 // TRUE
}

unsafe extern "system" fn entry_thread(_param: *mut c_void) -> u32 {
    // DLL 端 Patch 后通过共享内存 IPC 写入状态
    match crate::ipc::Ipc::new(crate::ipc::IpcMode::Write, false) {
        Ok(ipc) => {
            if startup() {
                ipc.data().success = true;
                #[cfg(debug_assertions)]
                crate::debuglog::clear();
            } else {
                ipc.data().success = false;
            }
            ipc.data().finished = true;
        }
        Err(_e) => {
            debug!("IPC failed: {_e}");
        }
    }
    #[cfg(debug_assertions)]
    crate::debuglog::write();

    // 注入完成后自动退出目标进程
    unsafe { ExitProcess(0) };
}

pub fn startup() -> bool {
    let ctx_path = crate::utils::path("ctx.bin");
    let ctx = match crate::ctx::Ctx::deserialize(ctx_path) {
        Some(ctx) => ctx,
        None => {
            return false;
        }
    };
    let av_name = ctx.name_str();
    if av_name.is_empty() {
        debug!("No AV Name");
        return false;
    }

    let bstr_name = BSTR::from(&av_name).as_ptr() as *mut u16;

    // 为下面注销注册初始化 COM 环境
    unsafe {
        let hr = CoInitialize(None);
        if hr.is_err() {
            debug!("CoInitialize failed: 0x{:x}", hr.0);
            return false;
        }
    }

    // 总是先注销
    let as_unreg_result = com::unregister_as_status();
    let av_unreg_result = com::unregister_av_status();

    // 如果 ctx.state == 0 (OFF)，只注销不注册
    if ctx.state == 0 {
        return as_unreg_result && av_unreg_result;
    }

    let as_result = com::register_as_status(bstr_name);
    let av_result = com::register_av_status(bstr_name);
    as_result && av_result
}

#[cfg(debug_assertions)]
mod debuglog {
    use std::cell::RefCell;
    use std::fs::OpenOptions;
    use std::io::Write;

    thread_local! {
        static LOG_BUF: RefCell<Vec<String>> = RefCell::new(Vec::new());
    }

    pub fn log(args: std::fmt::Arguments) {
        LOG_BUF.with(|buf| {
            buf.borrow_mut().push(format!("[Debug] {}", args));
        });
    }

    pub fn write() {
        LOG_BUF.with(|buf| {
            if !buf.borrow().is_empty() {
                if let Ok(mut file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("C:/Windows/Temp/defender-rs-log.txt")
                {
                    for line in buf.borrow().iter() {
                        let _ = writeln!(file, "{}", line);
                    }
                }
            }
        });
    }

    pub fn clear() {
        LOG_BUF.with(|buf| buf.borrow_mut().clear());
    }
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::debuglog::log(format_args!($($arg)*));
    };
}
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {};
}
