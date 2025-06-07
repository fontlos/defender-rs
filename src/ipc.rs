use windows::Win32::Foundation::*;
use windows::Win32::System::Memory::{
    CreateFileMappingA, FILE_MAP_ALL_ACCESS, FILE_MAP_READ, FILE_MAP_WRITE, MapViewOfFile,
    OpenFileMappingA, PAGE_READWRITE, UnmapViewOfFile,
};
use windows::core::PCSTR;

use std::ffi::CString;
use std::mem::size_of;
use std::ptr::null_mut;

pub const IPC_SEG_NAME: &str = "defender-disabler-ipc";

#[repr(u8)]
#[derive(PartialEq, Eq)]
pub enum IpcMode {
    #[allow(dead_code)]
    Read = 0,
    Write = 1,
    ReadWrite = 2,
}

#[repr(C)]
pub struct Data {
    pub finished: bool,
    pub success: bool,
}

pub struct Ipc {
    handle: HANDLE,
    data: *mut Data,
    mode: IpcMode,
    was_created: bool,
}

impl Ipc {
    pub fn new(mode: IpcMode, should_create: bool) -> Result<Self, String> {
        let flag = match mode {
            IpcMode::Read => FILE_MAP_READ,
            IpcMode::Write => FILE_MAP_WRITE,
            IpcMode::ReadWrite => FILE_MAP_ALL_ACCESS,
        };
        let name = CString::new(IPC_SEG_NAME).unwrap();
        let handle = unsafe {
            if should_create {
                CreateFileMappingA(
                    INVALID_HANDLE_VALUE,
                    None,
                    PAGE_READWRITE,
                    0,
                    size_of::<Data>() as u32,
                    PCSTR(name.as_ptr() as _),
                )
            } else {
                OpenFileMappingA(flag.0, false, PCSTR(name.as_ptr() as _))
            }
        }
        .map_err(|e| format!("Unable to access ipc seg: {e}"))?;

        if handle.is_invalid() {
            return Err("Unable to access ipc seg (invalid handle)".to_string());
        }

        let data_ptr = unsafe { MapViewOfFile(handle, flag, 0, 0, size_of::<Data>()) };
        if data_ptr.Value.is_null() {
            unsafe {
                CloseHandle(handle).unwrap();
            }
            return Err("Unable to map ipc".to_string());
        }
        let data = data_ptr.Value as *mut Data;

        Ok(Self {
            handle,
            data,
            mode,
            was_created: should_create,
        })
    }
    pub fn data(&self) -> &mut Data {
        unsafe { &mut *self.data }
    }
}

impl Drop for Ipc {
    fn drop(&mut self) {
        unsafe {
            if !self.data.is_null()
                && self.was_created
                && (self.mode == IpcMode::Write || self.mode == IpcMode::ReadWrite)
            {
                std::ptr::write_bytes(self.data, 0, 1);
            }
            if !self.data.is_null() {
                UnmapViewOfFile(std::mem::transmute(self.data)).ok();
                self.data = null_mut();
            }
            if !self.handle.is_invalid() {
                CloseHandle(self.handle).ok();
                self.handle = HANDLE(std::ptr::null_mut());
            }
        }
    }
}
