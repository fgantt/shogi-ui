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
      
      // Auto-register built-in engine if not present
      if !engine_storage.has_builtin_engine() {
        log::info!("Built-in engine not registered, registering now...");
        
        // Get the built-in engine path - try multiple locations
        let builtin_path_result = {
          let mut found_path: Option<String> = None;
          
          // Method 1: Try workspace root (development mode)
          if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
              // Navigate up to workspace root
              if let Some(workspace_root) = exe_dir.parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .and_then(|p| p.parent()) {
                let engine_path = workspace_root.join("target/release/shogi-engine");
                if engine_path.exists() {
                  found_path = Some(engine_path.display().to_string());
                  log::info!("Found engine via workspace root: {:?}", found_path);
                }
              }
              
              // Method 2: Try relative to executable
              if found_path.is_none() {
                let engine_path = exe_dir.join("../../../target/release/shogi-engine");
                if engine_path.exists() {
                  found_path = Some(engine_path.display().to_string());
                  log::info!("Found engine via relative path: {:?}", found_path);
                }
              }
            }
          }
          
          // Method 3: Try current directory
          if found_path.is_none() {
            if let Ok(current_dir) = std::env::current_dir() {
              let engine_path = current_dir.join("target/release/shogi-engine");
              if engine_path.exists() {
                found_path = Some(engine_path.display().to_string());
                log::info!("Found engine via current dir: {:?}", found_path);
              }
            }
          }
          
          found_path
        };

        if let Some(engine_path) = builtin_path_result {
          log::info!("Found built-in engine at: {}", engine_path);
          
          // Validate the engine
          let metadata = tauri::async_runtime::block_on(
            crate::engine_validator::validate_engine(&engine_path)
          ).ok();
          
          // Create config
          let config = crate::engine_storage::EngineConfig::new(
            "Built-in Engine".to_string(),
            engine_path,
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
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
