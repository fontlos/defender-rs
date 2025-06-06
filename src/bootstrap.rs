use crate::com;
use crate::ctx::Ctx;
use std::fs::OpenOptions;
use std::io::Write;

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

    if let Err(e) = com::init_com() {
        let _ = writeln!(file, "[defender-rs] CoInitialize failed: {e}");
        return false;
    }

    let bstr_name = crate::com::alloc_bstr_from_str(&av_name);

    let as_result = com::register_as_status(bstr_name, &mut file);
    let av_result = com::register_av_status(bstr_name, &mut file);
    let _ = writeln!(file, "[defender-rs] IWscASStatus result: {as_result:?}");
    let _ = writeln!(file, "[defender-rs] IWscAVStatus4 result: {av_result:?}");
    as_result.is_ok() && av_result.is_ok()
}
