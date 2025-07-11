mod args;
mod inject;
mod scm;
mod task;
mod wsc;

use windows::Win32::System::{
    Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize},
    Console::{ATTACH_PARENT_PROCESS, AllocConsole, AttachConsole, FreeConsole},
    SystemInformation::{
        OSVERSIONINFOEXA, VER_PRODUCT_TYPE, VerSetConditionMask, VerifyVersionInfoA,
    },
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

pub fn run() {
    // 首先设置一下环境
    let current_path = std::env::current_exe().unwrap();
    let current_dir = current_path.parent().unwrap();
    unsafe { std::env::set_var("DEFENDER_RS_PATH", current_dir.to_str().unwrap()) };
    // 解析命令行参数
    let args = args::Args::parse();

    if !args.auto {
        // 只在非 auto 模式下分配控制台
        unsafe {
            if AttachConsole(ATTACH_PARENT_PROCESS).is_err() {
                AllocConsole().expect("[Error]: Failed to allocate console");
            }
        }

        // 只在非 auto 模式下写入配置
        let mut ctx = Ctx::default_with_name(&args.name);
        if args.disable {
            ctx.state = 0; // OFF
        }
        let ctx_path = current_dir.join("ctx.bin");
        ctx.serialize(&ctx_path);
        println!("[Info]: Write context");
    }

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

    let ipc = match Ipc::new(IpcMode::ReadWrite, true) {
        Ok(ipc) => ipc,
        Err(e) => {
            eprintln!("[Error]: IPC: {}", e);
            return;
        }
    };
    ipc.data().finished = false;
    ipc.data().success = false;

    // 等待 DLL 端写入 finished
    println!("[Info]: Waiting for DLL inject...");

    inject::inject("defender_core.dll", "c:\\Windows\\System32\\Taskmgr.exe").unwrap();

    while !ipc.data().finished {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    if ipc.data().success {
        println!("[Info]: Inject succeeded");
    } else {
        eprintln!("[Error]: Inject failed");
        return;
    }

    // 防止进程意外中断, 添加或删除计划任务时, 我们在外面初始化, 注意下面成对的回收
    unsafe {
        CoInitializeEx(Some(std::ptr::null_mut()), COINIT_MULTITHREADED).unwrap();
    }

    // 默认情况下我们使用 on boot 的方式添加任务, 除非显式指定 `--on-login` 参数
    if !args.auto {
        let mode = if args.disable { "remove" } else { "add" };
        if let Err(e) = task::edit_task(args.disable, args.on_login) {
            eprintln!("[Error]: Failed to {} auto task: {}", mode, e);
        } else {
            println!("[Info]: Successfully {} auto task", mode);
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
        unsafe {
            FreeConsole().ok();
        }
    }
}
