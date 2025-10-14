mod commands;
mod engine_manager;
mod state;

use engine_manager::EngineManager;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Initialize engine manager
      let engine_manager = EngineManager::new(app.handle().clone());
      let app_state = AppState::new(engine_manager);

      // Store state
      app.manage(app_state);

      log::info!("Shogi Game backend initialized");

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::spawn_engine,
      commands::send_usi_command,
      commands::stop_engine,
      commands::get_engine_status,
      commands::list_engines,
      commands::stop_all_engines,
      commands::get_builtin_engine_path,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
