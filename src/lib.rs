mod bootstrap;
mod com;
mod ctx;
mod ipc;
pub mod loader;

use windows::Win32::{
    Foundation::{CloseHandle, HINSTANCE},
    System::Threading::{CreateThread, ExitProcess, THREAD_CREATION_FLAGS},
};

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
            ipc.data().success = crate::bootstrap::startup();
            ipc.data().finished = true;
        }
        Err(e) => {
            println!("[Error]: Failed to create IPC shared memory: {e}");
        }
    }
    // 注入完成后自动退出目标进程
    unsafe { ExitProcess(0) };
}
