mod commands;

use commands::SharedTask;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(SharedTask {
            status: Arc::new(Mutex::new(commands::TaskStatus::default())),
        })
        .invoke_handler(tauri::generate_handler![
            commands::run_task,
            commands::get_task_status,
            commands::get_tools,
            commands::get_agents,
            commands::confirm_action,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
