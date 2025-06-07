use std::ffi::c_void;
use std::io::Write;
use windows::Win32::System::Com::{CLSCTX_ALL, CoInitialize};
use windows::core::{GUID, HRESULT, PCWSTR};

// WSC接口相关GUID
const CLSID_WSC_ISV: GUID = GUID::from_values(
    0xF2102C37,
    0x90C3,
    0x450C,
    [0xB3, 0x0F6, 0x92, 0xBE, 0x16, 0x93, 0xBD, 0xF2],
);
const IID_IWSC_ASSTATUS: GUID = GUID::from_values(
    0x24E9756,
    0xBA6C,
    0x4AD1,
    [0x83, 0x21, 0x87, 0xBA, 0xE7, 0x8F, 0xD0, 0xE3],
);
const IID_IWSC_AVSTATUS4: GUID = GUID::from_values(
    0x4DCBAFAC,
    0x29BA,
    0x46B1,
    [0x80, 0xFC, 0xB8, 0xBD, 0xE3, 0xC0, 0xAE, 0x4D],
);

#[repr(C)]
pub struct IWscASStatus {
    pub lp_vtbl: *const IWscASStatusVtbl,
}

#[repr(C)]
pub struct IWscASStatusVtbl {
    pub parent: [usize; 7],
    pub register: unsafe extern "system" fn(
        this: *mut c_void,
        path: *mut u16,
        name: *mut u16,
        a: u32,
        b: u32,
    ) -> HRESULT,
    pub unregister: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub update_status:
        unsafe extern "system" fn(this: *mut c_void, state: u32, unk: i32) -> HRESULT,
}

#[repr(C)]
pub struct IWscAVStatus4 {
    pub lp_vtbl: *const IWscAVStatus4Vtbl,
}

#[repr(C)]
pub struct IWscAVStatus4Vtbl {
    pub query_interface: unsafe extern "system" fn(
        this: *mut c_void,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> HRESULT,
    pub add_ref: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub release: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub register_: unsafe extern "system" fn(
        this: *mut c_void,
        path: *mut u16,
        name: *mut u16,
        a: u32,
        b: u32,
    ) -> HRESULT,
    pub unregister: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub update_status:
        unsafe extern "system" fn(this: *mut c_void, state: u32, unk: i32) -> HRESULT,
    pub initiate_offline_cleaning:
        unsafe extern "system" fn(this: *mut c_void, a: *mut u16, b: *mut u16) -> HRESULT,
    pub notify_user_for_near_expiration:
        unsafe extern "system" fn(this: *mut c_void, a: u32) -> HRESULT,
    pub make_default_product_request: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub is_default_product_enforced:
        unsafe extern "system" fn(this: *mut c_void, result: *mut u32) -> HRESULT,
    pub update_scan_substatus: unsafe extern "system" fn(this: *mut c_void, status: u32) -> HRESULT,
    pub update_settings_substatus:
        unsafe extern "system" fn(this: *mut c_void, status: u32) -> HRESULT,
    pub update_protection_update_substatus:
        unsafe extern "system" fn(this: *mut c_void, status: u32) -> HRESULT,
    pub register_av: unsafe extern "system" fn(
        this: *mut c_void,
        a: *mut u16,
        b: *mut u16,
        c: u32,
        d: u32,
    ) -> HRESULT,
    pub unregister_av: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub update_status_av:
        unsafe extern "system" fn(this: *mut c_void, state: u32, unk: i32) -> HRESULT,
    pub initiate_offline_cleaning_av:
        unsafe extern "system" fn(this: *mut c_void, a: *mut u16, b: *mut u16) -> HRESULT,
    pub notify_user_for_near_expiration_av:
        unsafe extern "system" fn(this: *mut c_void, a: u32) -> HRESULT,
    pub register_fw: unsafe extern "system" fn(
        this: *mut c_void,
        a: *mut u16,
        b: *mut u16,
        c: u32,
        d: u32,
    ) -> HRESULT,
    pub unregister_fw: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub update_status_fw: unsafe extern "system" fn(this: *mut c_void, state: u32) -> HRESULT,
    pub register_as: unsafe extern "system" fn(
        this: *mut c_void,
        a: *mut u16,
        b: *mut u16,
        c: u32,
        d: u32,
    ) -> HRESULT,
    pub unregister_as: unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
    pub update_status_as:
        unsafe extern "system" fn(this: *mut c_void, state: u32, unk: i32) -> HRESULT,
}

unsafe extern "system" {
    fn SysAllocString(psz: PCWSTR) -> *mut u16;
}

pub fn init_com() -> Result<(), String> {
    unsafe {
        let hr = CoInitialize(None);
        if hr.is_err() {
            return Err(format!("CoInitialize failed: 0x{:x}", hr.0));
        }
    }
    Ok(())
}

// BSTR 分配
pub fn alloc_bstr_from_str(s: &str) -> *mut u16 {
    let wide: Vec<u16> = s.encode_utf16().chain(Some(0)).collect();
    unsafe { SysAllocString(PCWSTR(wide.as_ptr())) }
}

// 原生 CoCreateInstance FFI
unsafe fn cocreate_instance(rclsid: &GUID, riid: &GUID, ppv: *mut *mut c_void) -> HRESULT {
    #[link(name = "ole32")]
    unsafe extern "system" {
        fn CoCreateInstance(
            rclsid: *const GUID,
            pUnkOuter: *mut c_void,
            dwClsContext: u32,
            riid: *const GUID,
            ppv: *mut *mut c_void,
        ) -> HRESULT;
    }
    unsafe { CoCreateInstance(rclsid, std::ptr::null_mut(), CLSCTX_ALL.0, riid, ppv) }
}

pub fn register_as_status(name: *mut u16, file: &mut std::fs::File) -> Result<(), String> {
    unsafe {
        let hr_init = CoInitialize(None);
        let _ = writeln!(file, "[defender-rs][debug] CoInitialize: 0x{:x}", hr_init.0);
        if hr_init.is_err() {
            return Err(format!("CoInitialize failed: 0x{:x}", hr_init.0));
        }
        let mut obj: *mut c_void = std::ptr::null_mut();
        let hr = cocreate_instance(&CLSID_WSC_ISV, &IID_IWSC_ASSTATUS, &mut obj);
        let _ = writeln!(
            file,
            "[defender-rs][debug] CoCreateInstance: 0x{:x}, obj=0x{:x}",
            hr.0, obj as usize
        );
        if hr.0 < 0 || obj.is_null() {
            let _ = writeln!(
                file,
                "[defender-rs][error] CoCreateInstance IWscASStatus failed: 0x{:x}",
                hr.0
            );
            let _ = writeln!(
                file,
                "[defender-rs][debug] CLSID_WSC_ISV: {CLSID_WSC_ISV:?}"
            );
            let _ = writeln!(
                file,
                "[defender-rs][debug] IID_IWSC_ASSTATUS: {IID_IWSC_ASSTATUS:?}"
            );
            let _ = writeln!(file, "[defender-rs][debug] PID: {}", std::process::id());
            return Err(format!(
                "CoCreateInstance IWscASStatus failed: 0x{:x}",
                hr.0
            ));
        }
        let iface = obj as *mut IWscASStatus;
        let vtbl = (*iface).lp_vtbl;
        let bstr = name;
        let _ = writeln!(
            file,
            "[defender-rs][debug] IWscASStatus ptr: 0x{:x}, vtbl=0x{:x}",
            iface as usize, vtbl as usize
        );
        let hr = ((*vtbl).unregister)(iface as *mut _);
        let _ = writeln!(file, "[defender-rs] Unregister: 0x{:x}", hr.0);
        let hr = ((*vtbl).register)(iface as *mut _, bstr, bstr, 0, 0);
        let _ = writeln!(file, "[defender-rs] Register: 0x{:x}", hr.0);
        let hr = ((*vtbl).update_status)(iface as *mut _, 0, 1);
        let _ = writeln!(file, "[defender-rs] UpdateStatus: 0x{:x}", hr.0);
    }
    Ok(())
}

pub fn register_av_status(name: *mut u16, file: &mut std::fs::File) -> Result<(), String> {
    unsafe {
        let hr_init = CoInitialize(None);
        let _ = writeln!(file, "[defender-rs][debug] CoInitialize: 0x{:x}", hr_init.0);
        if hr_init.is_err() {
            return Err(format!("CoInitialize failed: 0x{:x}", hr_init.0));
        }
        let mut obj: *mut c_void = std::ptr::null_mut();
        let hr = cocreate_instance(&CLSID_WSC_ISV, &IID_IWSC_AVSTATUS4, &mut obj);
        let _ = writeln!(
            file,
            "[defender-rs][debug] CoCreateInstance: 0x{:x}, obj=0x{:x}",
            hr.0, obj as usize
        );
        if hr.0 < 0 || obj.is_null() {
            let _ = writeln!(
                file,
                "[defender-rs][error] CoCreateInstance IWscAVStatus4 failed: 0x{:x}",
                hr.0
            );
            let _ = writeln!(
                file,
                "[defender-rs][debug] CLSID_WSC_ISV: {CLSID_WSC_ISV:?}"
            );
            let _ = writeln!(
                file,
                "[defender-rs][debug] IID_IWSC_AVSTATUS4: {IID_IWSC_AVSTATUS4:?}"
            );
            let _ = writeln!(file, "[defender-rs][debug] PID: {}", std::process::id());
            return Err(format!(
                "CoCreateInstance IWscAVStatus4 failed: 0x{:x}",
                hr.0
            ));
        }
        let iface = obj as *mut IWscAVStatus4;
        let vtbl = (*iface).lp_vtbl;
        let bstr = name;
        let _ = writeln!(
            file,
            "[defender-rs][debug] IWscAVStatus4 ptr: 0x{:x}, vtbl=0x{:x}",
            iface as usize, vtbl as usize
        );
        let hr = ((*vtbl).unregister)(iface as *mut _);
        let _ = writeln!(file, "[defender-rs] Unregister: 0x{:x}", hr.0);
        let hr = ((*vtbl).register_)(iface as *mut _, bstr, bstr, 0, 0);
        let _ = writeln!(file, "[defender-rs] Register: 0x{:x}", hr.0);
        let hr = ((*vtbl).update_status)(iface as *mut _, 0, 1);
        let _ = writeln!(file, "[defender-rs] UpdateStatus: 0x{:x}", hr.0);
        let hr = ((*vtbl).update_scan_substatus)(iface as *mut _, 1);
        let _ = writeln!(file, "[defender-rs] UpdateScanSubstatus: 0x{:x}", hr.0);
        let hr = ((*vtbl).update_settings_substatus)(iface as *mut _, 1);
        let _ = writeln!(file, "[defender-rs] UpdateSettingsSubstatus: 0x{:x}", hr.0);
        let hr = ((*vtbl).update_protection_update_substatus)(iface as *mut _, 1);
        let _ = writeln!(
            file,
            "[defender-rs] UpdateProtectionUpdateSubstatus: 0x{:x}",
            hr.0
        );
    }
    Ok(())
}

pub fn unregister_as_status(_name: *mut u16, file: &mut std::fs::File) -> Result<(), String> {
    unsafe {
        let mut obj: *mut c_void = std::ptr::null_mut();
        let hr = cocreate_instance(&CLSID_WSC_ISV, &IID_IWSC_ASSTATUS, &mut obj);
        let _ = writeln!(
            file,
            "[defender-rs][debug] CoCreateInstance (AS) for unregister: 0x{:x}, obj=0x{:x}",
            hr.0, obj as usize
        );
        if hr.0 < 0 || obj.is_null() {
            return Err(format!(
                "CoCreateInstance IWscASStatus failed: 0x{:x}",
                hr.0
            ));
        }
        let iface = obj as *mut IWscASStatus;
        let vtbl = (*iface).lp_vtbl;
        let hr = ((*vtbl).unregister)(iface as *mut _);
        let _ = writeln!(file, "[defender-rs] Unregister (AS): 0x{:x}", hr.0);
        if hr.0 < 0 {
            return Err(format!("Unregister IWscASStatus failed: 0x{:x}", hr.0));
        }
    }
    Ok(())
}

pub fn unregister_av_status(_name: *mut u16, file: &mut std::fs::File) -> Result<(), String> {
    unsafe {
        let mut obj: *mut c_void = std::ptr::null_mut();
        let hr = cocreate_instance(&CLSID_WSC_ISV, &IID_IWSC_AVSTATUS4, &mut obj);
        let _ = writeln!(
            file,
            "[defender-rs][debug] CoCreateInstance (AV) for unregister: 0x{:x}, obj=0x{:x}",
            hr.0, obj as usize
        );
        if hr.0 < 0 || obj.is_null() {
            return Err(format!(
                "CoCreateInstance IWscAVStatus4 failed: 0x{:x}",
                hr.0
            ));
        }
        let iface = obj as *mut IWscAVStatus4;
        let vtbl = (*iface).lp_vtbl;
        let hr = ((*vtbl).unregister)(iface as *mut _);
        let _ = writeln!(file, "[defender-rs] Unregister (AV): 0x{:x}", hr.0);
        if hr.0 < 0 {
            return Err(format!("Unregister IWscAVStatus4 failed: 0x{:x}", hr.0));
        }
    }
    Ok(())
}
