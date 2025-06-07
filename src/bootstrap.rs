use windows::core::BSTR;
use windows::Win32::System::Com::CoInitialize;

use std::fs::OpenOptions;
use std::io::Write;

use crate::com;
use crate::ctx::Ctx;


pub fn startup() -> bool {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("D:/defender_rs_dll_loaded.log")
        .unwrap();
    let _ = writeln!(file, "[defender-rs] startup() called");

    let ctx = match Ctx::deserialize("ctx.bin") {
        Some(ctx) => ctx,
        None => {
            let _ = writeln!(file, "[defender-rs] ctx.bin not found or invalid");
            return false;
        }
    };
    let av_name = ctx.name_str();
    let _ = writeln!(file, "[defender-rs] AV Name: {}", av_name);
    if av_name.is_empty() {
        let _ = writeln!(file, "[defender-rs] AV Name is empty, aborting");
        return false;
    }

    unsafe {
        let hr = CoInitialize(None);
        if hr.is_err() {
            let _ = writeln!(file, "CoInitialize failed: 0x{:x}", hr.0);
            return false;
        }
    }

    let bstr_name = BSTR::from(&av_name).as_ptr() as *mut u16;

    // 总是先注销
    let as_unreg_result = com::unregister_as_status(bstr_name, &mut file);
    let av_unreg_result = com::unregister_av_status(bstr_name, &mut file);
    let _ = writeln!(
        file,
        "[defender-rs] IWscASStatus Unregister result: {as_unreg_result:?}"
    );
    let _ = writeln!(
        file,
        "[defender-rs] IWscAVStatus4 Unregister result: {av_unreg_result:?}"
    );

    // 如果 ctx.state == 0 (OFF)，只注销不注册
    if ctx.state == 0 {
        let _ = writeln!(file, "[defender-rs] ctx.state == 0, skip register");
        return true;
    }

    let as_result = com::register_as_status(bstr_name, &mut file);
    let av_result = com::register_av_status(bstr_name, &mut file);
    let _ = writeln!(file, "[defender-rs] IWscASStatus result: {as_result:?}");
    let _ = writeln!(file, "[defender-rs] IWscAVStatus4 result: {av_result:?}");
    as_result.is_ok() && av_result.is_ok()
}
