mod bootstrap;
mod com;
mod ctx;
mod ipc;
pub mod loader;

use std::ffi::c_void;
use std::sync::Once;

use windows::Win32::Foundation::{CloseHandle, HINSTANCE};
use windows::Win32::System::Threading::{CreateThread, THREAD_CREATION_FLAGS, ExitProcess};

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
    // DLL端：注册/patch后通过共享内存IPC写入状态，兼容C++ defendnot::InterProcessCommunication
    if let Ok(ipc) = crate::ipc::InterProcessCommunication::new(
        crate::ipc::InterProcessCommunicationMode::Write,
        false,
    ) {
        ipc.data().success = crate::bootstrap::startup();
        ipc.data().finished = true;
    } else {
        println!("[loader] Failed to create IPC shared memory");
    }
    // 注入完成后自动退出目标进程
    unsafe{ExitProcess(0)};
}
