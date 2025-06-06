mod bootstrap;
mod com;
mod ctx;
mod ipc;
pub mod loader;

use std::ffi::c_void;
use std::sync::Once;
use windows::Win32::Foundation::{CloseHandle, HINSTANCE};
use windows::Win32::System::Threading::{CreateThread, THREAD_CREATION_FLAGS};

static START: Once = Once::new();

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_hinst: HINSTANCE, reason: u32, _reserved: *mut c_void) -> i32 {
    const DLL_PROCESS_ATTACH: u32 = 1;
    if reason == DLL_PROCESS_ATTACH {
        START.call_once(|| {
            let h = unsafe {
                CreateThread(
                    None, // lpThreadAttributes
                    0,
                    Some(entry_thread),
                    None, // lpParameter
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
    // DLL端：注册/patch后通过共享内存IPC写入状态，兼容C++ defendnot::InterProcessCommunication
    if let Ok(ipc) = crate::ipc::InterProcessCommunication::new(
        crate::ipc::InterProcessCommunicationMode::Write,
        false,
    ) {
        // 2. 注册/patch主流程
        ipc.data().success = crate::bootstrap::startup();
        ipc.data().finished = true;
    } else {
        // 无法连接IPC，仍然执行主流程
        crate::bootstrap::startup();
    }
    0
}
