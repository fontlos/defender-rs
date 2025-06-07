mod args;
mod inject;
mod scm;
mod task;
mod wsc;

use windows::Win32::System::{
    Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize},
    Console::{ATTACH_PARENT_PROCESS, AllocConsole, AttachConsole, FreeConsole},
    SystemInformation::{
    VerSetConditionMask, VerifyVersionInfoA, VER_PRODUCT_TYPE,OSVERSIONINFOEXA
}
};

use crate::ctx::Ctx;
use crate::ipc::{Ipc, IpcMode};

pub fn is_winserver() -> bool {
    const VER_NT_WORKSTATION: u8 = 0x1;

    let mut osvi = OSVERSIONINFOEXA::default();
    osvi.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOEXA>() as u32;
    osvi.wProductType = VER_NT_WORKSTATION;
    let cond_mask = unsafe { VerSetConditionMask(0, VER_PRODUCT_TYPE, 1) };
    !unsafe { VerifyVersionInfoA(&mut osvi, VER_PRODUCT_TYPE, cond_mask).is_ok() }
}

fn alloc_console_if_needed() {
    unsafe {
        if AttachConsole(ATTACH_PARENT_PROCESS).is_err() {
            AllocConsole().expect("Failed to allocate console");
        }
    }
}

fn free_console_if_needed() {
    unsafe {
        FreeConsole().ok();
    }
}

pub fn run() {
    let args = args::Args::parse();
    if !args.auto {
        alloc_console_if_needed();
    }
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

    // 检测是否为 Windows Server
    if is_winserver() {
        println!("[Error]: Not support Windows Server");
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
    if !args.auto {
        match task::edit_task(args.disable, args.on_login) {
            Ok(_) => {
                println!("[Info] Successfully edit auto task");
            }
            Err(e) => {
                println!("[Error] Failed to edit auto task: {}", e);
            }
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

    if !args.auto {
        free_console_if_needed();
    }
}
