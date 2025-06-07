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

pub fn edit_task(disable: bool, on_login: bool) -> Result<(), String> {
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
            CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| format!("CoCreateInstance failed: {e:?}"))?;
        task_service
            .Connect(
                &VARIANT::default(),
                &VARIANT::default(),
                &VARIANT::default(),
                &VARIANT::default(),
            )
            .map_err(|e| format!("Connect failed: {e:?}"))?;

        // 获取 root 文件夹
        let root_folder = task_service
            .GetFolder(&BSTR::from("\\"))
            .map_err(|e| format!("GetFolder failed: {e:?}"))?;

        if disable {
            let _ = root_folder
                .DeleteTask(&BSTR::from(TASK_NAME), 0)
                .map_err(|e| format!("DeleteTask failed: {e:?}"))?;
            return Ok(());
        }

        // 创建 TaskDefinition
        let task_def = task_service
            .NewTask(0)
            .map_err(|e| format!("NewTask failed: {e:?}"))?;

        // 设置 Principal
        let principal = task_def
            .Principal()
            .map_err(|e| format!("Get Principal failed: {e:?}"))?;
        principal.SetLogonType(logon_type).ok();
        principal.SetRunLevel(TASK_RUNLEVEL_HIGHEST).ok();
        principal.SetUserId(&BSTR::from("SYSTEM")).ok();

        // 设置 Trigger
        let triggers = task_def
            .Triggers()
            .map_err(|e| format!("Get Triggers failed: {e:?}"))?;
        let _trigger = triggers
            .Create(trigger_type)
            .map_err(|e| format!("Create Trigger failed: {e:?}"))?;

        // 设置 Action
        let actions = task_def
            .Actions()
            .map_err(|e| format!("Get Actions failed: {e:?}"))?;
        let action = actions
            .Create(TASK_ACTION_EXEC)
            .map_err(|e| format!("Create Action failed: {e:?}"))?;
        let exec_action: IExecAction = action
            .cast()
            .map_err(|e| format!("Query IExecAction failed: {e:?}"))?;

        // 路径和参数
        let exe_path = std::env::current_exe().map_err(|e| format!("current_exe failed: {e:?}"))?;
        let exe_path_str = exe_path.to_str().ok_or("exe_path to_str failed")?;
        exec_action.SetPath(&BSTR::from(exe_path_str)).ok();
        exec_action.SetArguments(&BSTR::from(TASK_ARG)).ok();

        // 设置 Settings
        let settings = task_def
            .Settings()
            .map_err(|e| format!("Get Settings failed: {e:?}"))?;
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
            )
            .map_err(|e| format!("RegisterTaskDefinition failed: {e:?}"))?;

        Ok(())
    }
}
