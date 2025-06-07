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
pub struct IWscAVStatus4 {
    pub lp_vtbl: *const IWscAVStatus4Vtbl,
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

use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

const S_OK: HRESULT = HRESULT(0);
const E_NOTIMPL: HRESULT = HRESULT(0x80004001u32 as i32);

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

#[repr(C)]
pub struct IWscAVStatus4Impl {
    pub vtbl: *const IWscAVStatus4Vtbl,
    pub ref_count: AtomicU32,
}

// 所有方法实现
unsafe extern "system" fn av_register(
    _this: *mut c_void,
    _path: *mut u16,
    _name: *mut u16,
    _a: u32,
    _b: u32,
) -> HRESULT {
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(200));
    }
    S_OK
}
unsafe extern "system" fn av_unregister(_this: *mut c_void) -> HRESULT {
    S_OK
}
unsafe extern "system" fn av_update_status(_this: *mut c_void, _state: u32, _unk: i32) -> HRESULT {
    S_OK
}
unsafe extern "system" fn av_initiate_offline_cleaning(
    _this: *mut c_void,
    _a: *mut u16,
    _b: *mut u16,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn av_notify_user_for_near_expiration(
    _this: *mut c_void,
    _a: u32,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn av_make_default_product_request(_this: *mut c_void) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn av_is_default_product_enforced(
    _this: *mut c_void,
    result: *mut u32,
) -> HRESULT {
    unsafe {
        if !result.is_null() {
            *result = 0;
        }
    }
    S_OK
}
unsafe extern "system" fn av_update_scan_substatus(_this: *mut c_void, _status: u32) -> HRESULT {
    S_OK
}
unsafe extern "system" fn av_update_settings_substatus(
    _this: *mut c_void,
    _status: u32,
) -> HRESULT {
    S_OK
}
unsafe extern "system" fn av_update_protection_update_substatus(
    _this: *mut c_void,
    _status: u32,
) -> HRESULT {
    S_OK
}
unsafe extern "system" fn av_register_av(
    _this: *mut c_void,
    _a: *mut u16,
    _b: *mut u16,
    _c: u32,
    _d: u32,
) -> HRESULT {
    S_OK
}
unsafe extern "system" fn av_unregister_av(_this: *mut c_void) -> HRESULT {
    S_OK
}
unsafe extern "system" fn av_update_status_av(
    _this: *mut c_void,
    _state: u32,
    _unk: i32,
) -> HRESULT {
    S_OK
}
unsafe extern "system" fn av_initiate_offline_cleaning_av(
    _this: *mut c_void,
    _a: *mut u16,
    _b: *mut u16,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn av_notify_user_for_near_expiration_av(
    _this: *mut c_void,
    _a: u32,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn av_register_fw(
    _this: *mut c_void,
    _a: *mut u16,
    _b: *mut u16,
    _c: u32,
    _d: u32,
) -> HRESULT {
    S_OK
}
unsafe extern "system" fn av_unregister_fw(_this: *mut c_void) -> HRESULT {
    S_OK
}
unsafe extern "system" fn av_update_status_fw(_this: *mut c_void, _state: u32) -> HRESULT {
    S_OK
}
unsafe extern "system" fn av_register_as(
    _this: *mut c_void,
    _a: *mut u16,
    _b: *mut u16,
    _c: u32,
    _d: u32,
) -> HRESULT {
    S_OK
}
unsafe extern "system" fn av_unregister_as(_this: *mut c_void) -> HRESULT {
    S_OK
}
unsafe extern "system" fn av_update_status_as(
    _this: *mut c_void,
    _state: u32,
    _unk: i32,
) -> HRESULT {
    S_OK
}

// IUnknown 3 方法
unsafe extern "system" fn av_query_interface(
    this: *mut c_void,
    _riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    unsafe {
        if !ppv.is_null() {
            *ppv = this;
            av_add_ref(this);
            return S_OK;
        }
    }
    HRESULT(0x80004002u32 as i32)
}
unsafe extern "system" fn av_add_ref(this: *mut c_void) -> u32 {
    unsafe {
        let obj = this as *mut IWscAVStatus4Impl;
        (*obj).ref_count.fetch_add(1, Ordering::SeqCst) + 1
    }
}
unsafe extern "system" fn av_release(this: *mut c_void) -> u32 {
    unsafe {
        let obj = this as *mut IWscAVStatus4Impl;
        let count = (*obj).ref_count.fetch_sub(1, Ordering::SeqCst) - 1;
        if count == 0 {
            drop(Box::from_raw(obj));
        }
        count
    }
}

pub static IWSC_AVSTATUS4_VTBL: IWscAVStatus4Vtbl = IWscAVStatus4Vtbl {
    query_interface: av_query_interface,
    add_ref: av_add_ref,
    release: av_release,
    register_: av_register,
    unregister: av_unregister,
    update_status: av_update_status,
    initiate_offline_cleaning: av_initiate_offline_cleaning,
    notify_user_for_near_expiration: av_notify_user_for_near_expiration,
    make_default_product_request: av_make_default_product_request,
    is_default_product_enforced: av_is_default_product_enforced,
    update_scan_substatus: av_update_scan_substatus,
    update_settings_substatus: av_update_settings_substatus,
    update_protection_update_substatus: av_update_protection_update_substatus,
    register_av: av_register_av,
    unregister_av: av_unregister_av,
    update_status_av: av_update_status_av,
    initiate_offline_cleaning_av: av_initiate_offline_cleaning_av,
    notify_user_for_near_expiration_av: av_notify_user_for_near_expiration_av,
    register_fw: av_register_fw,
    unregister_fw: av_unregister_fw,
    update_status_fw: av_update_status_fw,
    register_as: av_register_as,
    unregister_as: av_unregister_as,
    update_status_as: av_update_status_as,
};

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
pub struct IWscASStatusImpl {
    pub vtbl: *const IWscASStatusVtbl,
    pub ref_count: AtomicU32,
}

unsafe extern "system" fn as_register(
    _this: *mut c_void,
    _path: *mut u16,
    _name: *mut u16,
    _a: u32,
    _b: u32,
) -> HRESULT {
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(200));
    }
    S_OK
}
unsafe extern "system" fn as_unregister(_this: *mut c_void) -> HRESULT {
    S_OK
}
unsafe extern "system" fn as_update_status(_this: *mut c_void, _state: u32, _unk: i32) -> HRESULT {
    S_OK
}

unsafe extern "system" fn as_query_interface(
    this: *mut c_void,
    _riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    unsafe {
        if !ppv.is_null() {
            *ppv = this;
            as_add_ref(this);
            return S_OK;
        }
    }
    HRESULT(0x80004002u32 as i32)
}
unsafe extern "system" fn as_add_ref(this: *mut c_void) -> u32 {
    unsafe {
        let obj = this as *mut IWscASStatusImpl;
        (*obj).ref_count.fetch_add(1, Ordering::SeqCst) + 1
    }
}
unsafe extern "system" fn as_release(this: *mut c_void) -> u32 {
    unsafe {
        let obj = this as *mut IWscASStatusImpl;
        let count = (*obj).ref_count.fetch_sub(1, Ordering::SeqCst) - 1;
        if count == 0 {
            drop(Box::from_raw(obj));
        }
        count
    }
}

pub static IWSC_ASSTATUS_VTBL: IWscASStatusVtbl = IWscASStatusVtbl {
    parent: [0; 7],
    register: as_register,
    unregister: as_unregister,
    update_status: as_update_status,
};

// 工厂函数, 供 DllGetClassObject 等调用
pub fn create_av_status4_object() -> *mut IWscAVStatus4Impl {
    let obj = Box::new(IWscAVStatus4Impl {
        vtbl: &IWSC_AVSTATUS4_VTBL,
        ref_count: AtomicU32::new(1),
    });
    Box::into_raw(obj)
}
pub fn create_as_status_object() -> *mut IWscASStatusImpl {
    let obj = Box::new(IWscASStatusImpl {
        vtbl: &IWSC_ASSTATUS_VTBL,
        ref_count: AtomicU32::new(1),
    });
    Box::into_raw(obj)
}
