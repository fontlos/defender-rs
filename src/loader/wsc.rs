use std::{thread, time::Duration};

use crate::loader::scm::{open_scm, open_service, query_service_running, start_service};

pub fn ensure_wsc() -> Result<(), String> {
    let scm = open_scm().ok_or("Unable to open Service Control Manager")?;
    let svc = open_service(scm, "wscsvc").ok_or("Unable to open wscsvc service")?;

    if query_service_running(svc) {
        return Ok(());
    }

    println!("[Info]: wscsvc is not running, starting it...");
    if !start_service(svc) {
        return Err("Failed to start wscsvc service".to_string());
    }

    println!("[Info]: Successfully started wscsvc, waiting for it to get up...");
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(500));
        if query_service_running(svc) {
            println!("[Info]: wscsvc is running, environment ready");
            return Ok(());
        }
    }
    Err("wscsvc service did not start in time".to_string())
}
