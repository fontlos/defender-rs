mod inject;
mod scm;
mod task;
mod wsc;

use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};

use crate::ctx::Ctx;
use crate::ipc::{Ipc, IpcMode};

pub struct Args {
    pub name: String,
    pub disable: bool,
    pub auto: bool,
    pub on_login: bool,
}

impl Args {
    pub fn parse() -> Self {
        let mut name = "Defender-rs".to_string();
        let mut disable = false;
        let mut auto = false;
        let mut on_login = false;
        let mut args = std::env::args().skip(1); // 跳过程序名

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--name" => {
                    if let Some(val) = args.next() {
                        name = val;
                    }
                }
                "--disable" => {
                    disable = true;
                }
                "--auto" => {
                    auto = true;
                }
                "--on-login" => {
                    on_login = true;
                }
                _ => {}
            }
        }

        Args {
            name,
            disable,
            auto,
            on_login,
        }
    }
}

pub fn run() {
    let args = Args::parse();

    let mut ctx = Ctx::default_with_name(&args.name);
    if args.disable {
        ctx.state = 0; // OFF
    }
    ctx.serialize("ctx.bin");
    println!("[Info]: Write context");

    // 环境准备, 确保 wscsvc 服务已启动
    if let Err(e) = wsc::ensure_wsc() {
        eprintln!("[Error]: WSC: {}", e);
        return;
    }

    let ipc = Ipc::new(IpcMode::ReadWrite, true).expect("Failed to create IPC shared memory");
    ipc.data().finished = false;
    ipc.data().success = false;

    // 等待 DLL 端写入 finished
    println!("[Info]: Waiting for DLL inject...");

    inject::inject("defender_rs.dll", "c:\\Windows\\System32\\Taskmgr.exe").unwrap();

    while !ipc.data().finished {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!(
        "[Info] Inject DLL: {}, success: {}",
        ipc.data().finished,
        ipc.data().success
    );

    // 防止进程意外中断, 添加或删除计划任务时, 我们在外面初始化, 注意下面成对的回收
    unsafe {
        CoInitializeEx(Some(std::ptr::null_mut()), COINIT_MULTITHREADED).unwrap();
    }

    // 默认情况下我们使用 on boot 的方式添加任务, 除非显式指定 `--on-login` 参数
    match task::add_task(args.on_login) {
        Ok(_) => {
            println!("[Info] Successfully add auto task");
        }
        Err(e) => {
            println!("[Error] Failed to add auto task: {}", e);
        }
    }

    // 防止 loader 进程提前退出
    use std::io::{self, Write};
    println!("Press Enter to exit...");
    let _ = io::stdout().flush();
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
    unsafe {
        CoUninitialize();
    }
}
