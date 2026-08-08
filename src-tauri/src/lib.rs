mod models;
mod optimizer;
mod queue;
mod scanner;
mod settings;

use queue::AppState;
use settings::SettingsStore;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let config_dir = app.path().app_config_dir()?;
            app.manage(AppState::new(SettingsStore::new(
                config_dir.join("settings.json"),
            )));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            queue::add_paths,
            queue::list_jobs,
            queue::cancel_jobs,
            queue::retry_jobs,
            queue::clear_jobs,
            queue::load_settings,
            queue::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PicoPic");
}
