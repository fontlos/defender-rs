mod com;
mod ctx;
mod ipc;
pub mod loader;

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
            ipc.data().success = startup();
            ipc.data().finished = true;
        }
        Err(e) => {
            println!("[Error]: Failed to create IPC shared memory: {e}");
        }
    }
    // 注入完成后自动退出目标进程
    unsafe { ExitProcess(0) };
}

pub fn startup() -> bool {
    let ctx = match crate::ctx::Ctx::deserialize("ctx.bin") {
        Some(ctx) => ctx,
        None => {
            debug!("[defender-rs] ctx.bin not found or invalid");
            return false;
        }
    };
    let av_name = ctx.name_str();
    if av_name.is_empty() {
        debug!("[defender-rs] AV Name is empty, aborting");
        return false;
    }

    unsafe {
        let hr = CoInitialize(None);
        if hr.is_err() {
            debug!("[defender-rs] CoInitialize failed: 0x{:x}", hr.0);
            return false;
        }
    }

    let bstr_name = BSTR::from(&av_name).as_ptr() as *mut u16;

    // 总是先注销
    let as_unreg_result = com::unregister_as_status(bstr_name);
    let av_unreg_result = com::unregister_av_status(bstr_name);

    // 如果 ctx.state == 0 (OFF)，只注销不注册
    if ctx.state == 0 {
        debug!("[defender-rs] IWscASStatus Unregister result: {as_unreg_result:?}");
        debug!("[defender-rs] IWscAVStatus4 Unregister result: {av_unreg_result:?}");
        return as_unreg_result.is_ok() && av_unreg_result.is_ok();
    }

    let as_result = com::register_as_status(bstr_name);
    let av_result = com::register_av_status(bstr_name);
    debug!("[defender-rs] IWscASStatus result: {as_result:?}");
    debug!("[defender-rs] IWscAVStatus4 result: {av_result:?}");
    as_result.is_ok() && av_result.is_ok()
}

#[cfg(debug_assertions)]
mod debuglog {
    use std::cell::RefCell;
    use std::fs::{File, OpenOptions};
    use std::io::Write;

    thread_local! {
        static LOG_FILE: RefCell<Option<File>> = RefCell::new(None);
    }

    pub fn log(args: std::fmt::Arguments) {
        LOG_FILE.with(|cell| {
            let mut opt = cell.borrow_mut();
            if opt.is_none() {
                *opt = Some(
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("D:\\defender-rs-log.txt")
                        .unwrap(),
                );
            }
            if let Some(file) = opt.as_mut() {
                let _ = file.write_fmt(args);
                let _ = file.write_all(b"\n");
            }
        });
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
