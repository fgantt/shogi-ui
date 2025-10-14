mod commands;
mod engine_manager;
mod engine_storage;
mod engine_validator;
mod state;

use engine_manager::EngineManager;
use engine_storage::EngineStorage;
use state::AppState;
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

      // Initialize engine manager
      let engine_manager = EngineManager::new(app.handle().clone());
      
      // Load engine storage
      let engine_storage = match tauri::async_runtime::block_on(EngineStorage::load()) {
        Ok(storage) => storage,
        Err(e) => {
          log::error!("Failed to load engine storage: {}", e);
          EngineStorage::default()
        }
      };
      
      let app_state = AppState::new(engine_manager, engine_storage);

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
      commands::add_engine,
      commands::remove_engine,
      commands::get_engines,
      commands::validate_engine_path,
      commands::register_builtin_engine,
      commands::health_check_engines,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
