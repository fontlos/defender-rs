use windows::Win32::{
    Foundation::VARIANT_FALSE,
    System::{
        Com::{CLSCTX_INPROC_SERVER, CoCreateInstance},
        TaskScheduler::{
            IExecAction, ITaskService, TASK_ACTION_EXEC, TASK_CREATE_OR_UPDATE, TASK_LOGON_NONE,
            TASK_LOGON_SERVICE_ACCOUNT, TASK_RUNLEVEL_HIGHEST, TASK_TRIGGER_BOOT,
            TASK_TRIGGER_LOGON, TaskScheduler,
        },
        Variant::VARIANT,
    },
};
use windows::core::{BSTR, Interface};

pub fn edit_task(disable: bool, on_login: bool) -> windows::core::Result<()> {
    const TASK_NAME: &str = "Defender-rs";
    const TASK_ARG: &str = "--auto";
    let (trigger_type, logon_type) = if !on_login {
        (TASK_TRIGGER_BOOT, TASK_LOGON_SERVICE_ACCOUNT)
    } else {
        (TASK_TRIGGER_LOGON, TASK_LOGON_NONE)
    };

    unsafe {
        // 创建 TaskService
        let task_service: ITaskService =
            CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;
        task_service
            .Connect(
                &VARIANT::default(),
                &VARIANT::default(),
                &VARIANT::default(),
                &VARIANT::default(),
            )?;

        // 获取 root 文件夹
        let root_folder = task_service
            .GetFolder(&BSTR::from("\\"))?;

        if disable {
            let _ = root_folder
                .DeleteTask(&BSTR::from(TASK_NAME), 0)?;
            return Ok(());
        }

        // 创建 TaskDefinition
        let task_def = task_service
            .NewTask(0)?;

        // 设置 Principal
        let principal = task_def
            .Principal()?;
        principal.SetLogonType(logon_type).ok();
        principal.SetRunLevel(TASK_RUNLEVEL_HIGHEST).ok();
        principal.SetUserId(&BSTR::from("SYSTEM")).ok();

        // 设置 Trigger
        let triggers = task_def
            .Triggers()?;
        let _trigger = triggers
            .Create(trigger_type)?;

        // 设置 Action
        let actions = task_def
            .Actions()?;
        let action = actions
            .Create(TASK_ACTION_EXEC)?;
        let exec_action: IExecAction = action
            .cast()?;

        // 路径和参数
        let exe_path = std::env::current_exe().unwrap();
        let exe_path_str = exe_path.to_str().unwrap();
        exec_action.SetPath(&BSTR::from(exe_path_str)).ok();
        exec_action.SetArguments(&BSTR::from(TASK_ARG)).ok();

        // 设置 Settings
        let settings = task_def
            .Settings()?;
        settings.SetDisallowStartIfOnBatteries(VARIANT_FALSE).ok();
        settings.SetStopIfGoingOnBatteries(VARIANT_FALSE).ok();

        // 注册任务
        let _ = root_folder
            .RegisterTaskDefinition(
                &BSTR::from(TASK_NAME),
                &task_def,
                TASK_CREATE_OR_UPDATE.0,
                &VARIANT::default(),
                &VARIANT::default(),
                TASK_LOGON_NONE,
                &VARIANT::default(),
            )?;

        Ok(())
    }
}
