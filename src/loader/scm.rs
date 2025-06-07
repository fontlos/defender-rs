use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::Win32::{
    Foundation::{ERROR_SERVICE_ALREADY_RUNNING, GetLastError},
    System::Services::{
        CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx, SC_HANDLE,
        SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO, SERVICE_QUERY_STATUS, SERVICE_START,
        SERVICE_STATUS_PROCESS, StartServiceW,
    },
};
use windows::core::PCWSTR;

pub struct ServiceHandle(pub SC_HANDLE);
impl Drop for ServiceHandle {
    fn drop(&mut self) {
        unsafe {
            CloseServiceHandle(self.0).ok();
        }
    }
}

pub fn open_scm() -> Option<SC_HANDLE> {
    unsafe {
        let h = OpenSCManagerW(None, None, SC_MANAGER_CONNECT).unwrap();
        if h.0 == std::ptr::null_mut() {
            None
        } else {
            Some(h)
        }
    }
}

pub fn open_service(scm: SC_HANDLE, name: &str) -> Option<SC_HANDLE> {
    let name_w: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0)).collect();
    unsafe {
        let h = OpenServiceW(
            scm,
            PCWSTR(name_w.as_ptr()),
            SERVICE_QUERY_STATUS | SERVICE_START,
        )
        .unwrap();
        if h.0 == std::ptr::null_mut() {
            None
        } else {
            Some(h)
        }
    }
}

pub fn query_service_running(svc: SC_HANDLE) -> bool {
    let mut status = SERVICE_STATUS_PROCESS::default();
    let status_bytes = unsafe {
        std::slice::from_raw_parts_mut(
            &mut status as *mut SERVICE_STATUS_PROCESS as *mut u8,
            std::mem::size_of::<SERVICE_STATUS_PROCESS>(),
        )
    };

    let mut needed = 0u32;
    let ok = unsafe {
        QueryServiceStatusEx(
            svc,
            SC_STATUS_PROCESS_INFO,
            Some(status_bytes),
            &mut needed as *mut u32,
        )
        .is_ok()
    };
    if !ok {
        return false;
    }
    status.dwCurrentState.0 == 0x00000004
}

pub fn start_service(svc: SC_HANDLE) -> bool {
    unsafe { StartServiceW(svc, None).is_ok() || GetLastError() == ERROR_SERVICE_ALREADY_RUNNING }
}
