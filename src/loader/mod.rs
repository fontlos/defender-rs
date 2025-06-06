pub mod inject;

use crate::ipc::{InterProcessCommunication, InterProcessCommunicationMode};

use crate::ctx::Ctx;

pub fn run() {
    pub const DEFAULT_AV_NAME: &str = "Defender-rs";
    let ctx = Ctx::default_with_name(DEFAULT_AV_NAME);
    ctx.serialize("ctx.bin");
    println!("[loader] Context init");

    let ipc = InterProcessCommunication::new(InterProcessCommunicationMode::ReadWrite, true)
        .expect("Failed to create IPC shared memory");
    ipc.data().finished = false;
    ipc.data().success = false;

    // 等待 DLL 端写入 finished
    println!("[loader] Waiting for DLL to signal finished...");

    inject::inject("defender_rs.dll", "c:\\Windows\\System32\\Taskmgr.exe").unwrap();

    while !ipc.data().finished {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!(
        "[loader] DLL finished: {}, success: {}",
        ipc.data().finished,
        ipc.data().success
    );

    // 阻塞等待用户输入，防止 loader 进程提前退出
    use std::io::{self, Write};
    println!("按回车键退出...");
    let _ = io::stdout().flush();
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
}
