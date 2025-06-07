use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    Security::SECURITY_ATTRIBUTES,
    System::{
        Diagnostics::Debug::{
            DebugActiveProcessStop, DebugSetProcessKillOnExit, WriteProcessMemory,
        },
        LibraryLoader::{GetModuleHandleA, GetProcAddress},
        Memory::{
            MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx, VirtualFreeEx,
        },
        Threading::{
            CREATE_SUSPENDED, CreateProcessA, CreateRemoteThread, DEBUG_ONLY_THIS_PROCESS,
            DEBUG_PROCESS, INFINITE, PROCESS_INFORMATION, STARTUPINFOA, WaitForSingleObject,
        },
    },
};
use windows::core::{PCSTR, PSTR};

use std::ffi::CString;

pub fn inject(dll_path: &str, proc_name: &str) -> windows::core::Result<HANDLE> {
    let mut si = STARTUPINFOA::default();
    si.cb = std::mem::size_of::<STARTUPINFOA>() as u32;
    let mut pi = PROCESS_INFORMATION::default();
    let mut sa = SECURITY_ATTRIBUTES::default();
    sa.nLength = std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32;
    sa.bInheritHandle = true.into();

    // 启动进程, 挂起 + 调试
    let process_flags = CREATE_SUSPENDED | DEBUG_PROCESS | DEBUG_ONLY_THIS_PROCESS;
    let proc_name_c = CString::new(proc_name).unwrap();
    let res = unsafe {
        CreateProcessA(
            None,
            Some(PSTR(proc_name_c.as_ptr() as *mut u8)),
            Some(&mut sa),
            Some(&mut sa),
            false,
            process_flags,
            None,
            None,
            &mut si,
            &mut pi,
        )
    };
    if res.is_err() {
        return Err(windows::core::Error::from_win32());
    }

    // 分离调试器
    unsafe {
        DebugSetProcessKillOnExit(false).unwrap();
        DebugActiveProcessStop(pi.dwProcessId).unwrap();
    }

    // 写入 DLL 路径
    let dll_path_c = CString::new(dll_path).unwrap();
    let mem = unsafe {
        VirtualAllocEx(
            pi.hProcess,
            None,
            dll_path_c.as_bytes_with_nul().len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    };
    if mem.is_null() {
        unsafe {
            CloseHandle(pi.hThread).unwrap();
        }
        return Err(windows::core::Error::from_win32());
    }

    let write_ok = unsafe {
        WriteProcessMemory(
            pi.hProcess,
            mem,
            dll_path_c.as_ptr() as _,
            dll_path_c.as_bytes_with_nul().len(),
            None,
        )
    };
    if write_ok.is_err() {
        unsafe {
            VirtualFreeEx(pi.hProcess, mem, 0, MEM_RELEASE).unwrap();
            CloseHandle(pi.hThread).unwrap();
        }
        return Err(windows::core::Error::from_win32());
    }

    // 远程线程 LoadLibraryA
    let h_kernel32 = unsafe { GetModuleHandleA(PCSTR(b"kernel32.dll\0".as_ptr())).unwrap() };
    let load_library =
        unsafe { GetProcAddress(h_kernel32, PCSTR(b"LoadLibraryA\0".as_ptr())).unwrap() };
    let thread = unsafe {
        CreateRemoteThread(
            pi.hProcess,
            None,
            0,
            Some(std::mem::transmute(load_library)),
            Some(mem),
            0,
            None,
        )
        .unwrap()
    };

    unsafe {
        WaitForSingleObject(thread, INFINITE);
        CloseHandle(thread).unwrap();
        VirtualFreeEx(pi.hProcess, mem, 0, MEM_RELEASE).unwrap();
        CloseHandle(pi.hThread).unwrap();
    }
    Ok(pi.hProcess)
}
