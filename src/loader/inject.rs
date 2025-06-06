use std::ffi::CString;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::System::Diagnostics::Debug::{
    DebugActiveProcessStop, DebugSetProcessKillOnExit, WriteProcessMemory,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx, VirtualFreeEx,
};
use windows::Win32::System::Threading::{
    CREATE_SUSPENDED, CreateProcessA, CreateRemoteThread, DEBUG_ONLY_THIS_PROCESS, DEBUG_PROCESS,
    INFINITE, PROCESS_INFORMATION, STARTUPINFOA, WaitForSingleObject,
};
use windows::core::{PCSTR, PSTR};

pub fn inject(dll_path: &str, proc_name: &str) -> windows::core::Result<HANDLE> {
    unsafe {
        let mut si = STARTUPINFOA::default();
        si.cb = std::mem::size_of::<STARTUPINFOA>() as u32;
        let mut pi = PROCESS_INFORMATION::default();
        let mut sa = SECURITY_ATTRIBUTES::default();
        sa.nLength = std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32;
        sa.bInheritHandle = true.into();

        // 启动进程（挂起+调试）
        let process_flags = CREATE_SUSPENDED | DEBUG_PROCESS | DEBUG_ONLY_THIS_PROCESS;
        let proc_name_c = CString::new(proc_name).unwrap();
        let res = CreateProcessA(
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
        );
        if res.is_err() {
            return Err(windows::core::Error::from_win32());
        }

        // Detach
        DebugSetProcessKillOnExit(false).unwrap();
        DebugActiveProcessStop(pi.dwProcessId).unwrap();

        // 写入DLL路径
        let dll_path_c = CString::new(dll_path).unwrap();
        let mem = VirtualAllocEx(
            pi.hProcess,
            None,
            dll_path_c.as_bytes_with_nul().len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        if mem.is_null() {
            CloseHandle(pi.hThread).unwrap();
            return Err(windows::core::Error::from_win32());
        }

        let write_ok = WriteProcessMemory(
            pi.hProcess,
            mem,
            dll_path_c.as_ptr() as _,
            dll_path_c.as_bytes_with_nul().len(),
            None,
        );
        if write_ok.is_err() {
            VirtualFreeEx(pi.hProcess, mem, 0, MEM_RELEASE).unwrap();
            CloseHandle(pi.hThread).unwrap();
            return Err(windows::core::Error::from_win32());
        }

        // 远程线程LoadLibraryA
        let h_kernel32 = GetModuleHandleA(PCSTR(b"kernel32.dll\0".as_ptr())).unwrap();
        let load_library = GetProcAddress(h_kernel32, PCSTR(b"LoadLibraryA\0".as_ptr())).unwrap();
        let thread = CreateRemoteThread(
            pi.hProcess,
            None,
            0,
            Some(std::mem::transmute(load_library)),
            Some(mem),
            0,
            None,
        )
        .unwrap();

        WaitForSingleObject(thread, INFINITE);
        CloseHandle(thread).unwrap();
        VirtualFreeEx(pi.hProcess, mem, 0, MEM_RELEASE).unwrap();
        CloseHandle(pi.hThread).unwrap();
        Ok(pi.hProcess)
    }
}
