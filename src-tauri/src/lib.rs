mod commands;
mod engine_manager;
mod engine_storage;
mod engine_validator;
mod engine_vs_engine;
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
      let mut engine_storage = match tauri::async_runtime::block_on(EngineStorage::load()) {
        Ok(storage) => storage,
        Err(e) => {
          log::error!("Failed to load engine storage: {}", e);
          EngineStorage::default()
        }
      };
      
      // Auto-register built-in engine if not present, or fix path if it's incorrect
      // Get the correct path first
      let correct_path = if cfg!(debug_assertions) {
        commands::find_workspace_root()
          .map(|workspace_root| workspace_root.join("target/release/usi-engine"))
          .filter(|engine_path| engine_path.exists())
          .map(|engine_path| engine_path.display().to_string())
      } else {
        None
      };
      
      if !engine_storage.has_builtin_engine() {
        log::info!("Built-in engine not registered, registering now...");
        
        if let Some(engine_path) = correct_path.as_ref() {
          log::info!("Found built-in engine at: {}", engine_path);
          
          // Validate the engine
          let metadata = tauri::async_runtime::block_on(
            crate::engine_validator::validate_engine(&engine_path)
          ).ok();
          
          // Create config
          let config = crate::engine_storage::EngineConfig::new(
            "Built-in Engine".to_string(),
            engine_path.clone(),
            metadata,
            true,
          );
          
          // Add to storage
          if let Ok(_) = engine_storage.add_engine(config) {
            // Save to disk
            if let Err(e) = tauri::async_runtime::block_on(engine_storage.save()) {
              log::error!("Failed to save engine storage: {}", e);
            } else {
              log::info!("Built-in engine registered successfully");
            }
          }
        } else {
          log::warn!("Could not find built-in engine executable");
        }
      } else if let Some(correct_path) = correct_path.as_ref() {
        // Builtin engine exists - check if path needs updating
        if let Some(builtin_engine) = engine_storage.engines.iter_mut().find(|e| e.is_builtin) {
          let path_exists = std::path::Path::new(&builtin_engine.path).exists();
          let path_is_correct = builtin_engine.path == *correct_path;
          
          if !path_is_correct || !path_exists {
            log::info!("Updating built-in engine path from '{}' to '{}'", builtin_engine.path, correct_path);
            builtin_engine.path = correct_path.clone();
            
            // Validate the engine and update metadata
            let metadata = tauri::async_runtime::block_on(
              crate::engine_validator::validate_engine(correct_path)
            ).ok();
            builtin_engine.metadata = metadata;
            
            // Save to disk
            if let Err(e) = tauri::async_runtime::block_on(engine_storage.save()) {
              log::error!("Failed to save engine storage: {}", e);
            } else {
              log::info!("Built-in engine path updated successfully");
            }
          } else {
            log::info!("Built-in engine already has correct path: {}", correct_path);
          }
        }
      }
      
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
      commands::start_engine_vs_engine,
      commands::save_engine_options,
      commands::get_engine_options,
      commands::clone_engine,
      commands::update_engine_display_name,
      commands::set_favorite_engine,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
