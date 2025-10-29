use crate::engine_manager::EngineStatus;
use crate::engine_storage::EngineConfig;
use crate::engine_validator;
use crate::engine_vs_engine::{EngineVsEngineConfig, EngineVsEngineManager};
use crate::state::AppState;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub status: EngineStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}

impl CommandResponse {
    pub fn success() -> Self {
        Self {
            success: true,
            message: None,
            data: None,
        }
    }

    pub fn success_with_data(data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: None,
            data: Some(data),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message: Some(message),
            data: None,
        }
    }
}

/// Spawn a new USI engine process
#[tauri::command]
pub async fn spawn_engine(
    engine_id: String,
    name: String,
    path: String,
    temp_options: Option<std::collections::HashMap<String, String>>,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::info!("Command: spawn_engine - id: {}, name: {}, path: {}", engine_id, name, path);
    if let Some(ref opts) = temp_options {
        log::info!("Using {} temporary options for this game", opts.len());
    }

    let manager = &state.engine_manager;
    
    match manager.spawn_engine(engine_id.clone(), name, path).await {
        Ok(_) => {
            // Initialize the engine with USI protocol and send options
            // Use temp_options if provided, otherwise use saved options from storage
            if let Err(e) = manager.initialize_engine_with_temp_options(
                &engine_id, 
                &state.engine_storage,
                temp_options.as_ref()
            ).await {
                log::error!("Failed to initialize engine: {}", e);
                let _ = manager.stop_engine(&engine_id).await;
                return Ok(CommandResponse::error(format!("Failed to initialize engine: {}", e)));
            }
            
            Ok(CommandResponse::success_with_data(
                serde_json::json!({ "engine_id": engine_id })
            ))
        }
        Err(e) => {
            log::error!("Failed to spawn engine: {}", e);
            Ok(CommandResponse::error(format!("Failed to spawn engine: {}", e)))
        }
    }
}

/// Send a USI command to a specific engine
#[tauri::command]
pub async fn send_usi_command(
    engine_id: String,
    command: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::debug!("Command: send_usi_command - engine_id: {}, command: {}", engine_id, command);

    let manager = &state.engine_manager;

    match manager.send_command(&engine_id, &command).await {
        Ok(_) => Ok(CommandResponse::success()),
        Err(e) => {
            log::error!("Failed to send command to engine: {}", e);
            Ok(CommandResponse::error(format!("Failed to send command: {}", e)))
        }
    }
}

/// Stop a specific engine
#[tauri::command]
pub async fn stop_engine(
    engine_id: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::info!("Command: stop_engine - engine_id: {}", engine_id);

    let manager = &state.engine_manager;

    match manager.stop_engine(&engine_id).await {
        Ok(_) => Ok(CommandResponse::success()),
        Err(e) => {
            log::error!("Failed to stop engine: {}", e);
            Ok(CommandResponse::error(format!("Failed to stop engine: {}", e)))
        }
    }
}

/// Get the status of a specific engine
#[tauri::command]
pub async fn get_engine_status(
    engine_id: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    let manager = &state.engine_manager;

    match manager.get_engine_status(&engine_id).await {
        Some(status) => Ok(CommandResponse::success_with_data(
            serde_json::json!({ "status": status })
        )),
        None => Ok(CommandResponse::error("Engine not found".to_string())),
    }
}

/// List all active engines
#[tauri::command]
pub async fn list_engines(
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    let manager = &state.engine_manager;
    let engine_ids = manager.list_engines().await;

    Ok(CommandResponse::success_with_data(
        serde_json::json!({ "engines": engine_ids })
    ))
}

/// Stop all engines (cleanup)
#[tauri::command]
pub async fn stop_all_engines(
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::info!("Command: stop_all_engines");

    let manager = &state.engine_manager;

    match manager.stop_all_engines().await {
        Ok(_) => Ok(CommandResponse::success()),
        Err(e) => {
            log::error!("Failed to stop all engines: {}", e);
            Ok(CommandResponse::error(format!("Failed to stop all engines: {}", e)))
        }
    }
}

/// Helper function to find the workspace root by looking for the root Cargo.toml
/// that defines the usi-engine binary
pub fn find_workspace_root() -> Option<std::path::PathBuf> {
    // Start from current directory or executable location
    let start_dir = std::env::current_dir().ok()
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        })?;
    
    // Walk up from current directory to find workspace root
    let mut current = start_dir.as_path();
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            // Check if this Cargo.toml defines the usi-engine binary
            if let Ok(contents) = std::fs::read_to_string(&cargo_toml) {
                // Look for [[bin]] section with name = "usi-engine"
                // The pattern should be [[bin]] followed by name = "usi-engine" (usually on next line)
                let has_bin_def = if contents.contains("[[bin]]") {
                    // Check if "name = \"usi-engine\"" appears near a [[bin]] declaration
                    // Try to find [[bin]] and then check within next 10 lines
                    let lines: Vec<&str> = contents.lines().collect();
                    for (i, line) in lines.iter().enumerate() {
                        if line.trim() == "[[bin]]" && i + 1 < lines.len() {
                            // Check next few lines for name = "usi-engine"
                            for j in (i + 1)..std::cmp::min(i + 5, lines.len()) {
                                if lines[j].contains("name = \"usi-engine\"") || lines[j].contains("name = 'usi-engine'") {
                                    return Some(current.to_path_buf());
                                }
                            }
                        }
                    }
                    false
                } else {
                    false
                };
                
                if has_bin_def {
                    // Found the root Cargo.toml with usi-engine definition
                    return Some(current.to_path_buf());
                }
            }
        }
        
        // Check if we're at the filesystem root
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }
    
    None
}

/// Get the path to the bundled built-in engine
#[tauri::command]
pub async fn get_builtin_engine_path(
    app_handle: tauri::AppHandle,
) -> Result<CommandResponse, String> {
    use tauri::Manager;

    if cfg!(debug_assertions) {
        // Development mode - find the engine in the workspace
        let workspace_root = find_workspace_root()
            .ok_or_else(|| "Could not find workspace root".to_string())?;
        
        let engine_path = workspace_root.join("target/release/usi-engine");
        let engine_path_str = engine_path.display().to_string();
        
        if !engine_path.exists() {
            log::warn!("Engine not found at: {}. Attempting to build it...", engine_path_str);
            return Ok(CommandResponse::error(format!(
                "Engine not found at: {}. Please run 'cargo build --bin usi-engine --release' first.",
                engine_path_str
            )));
        }
        
        log::info!("Built-in engine path: {}", engine_path_str);
        return Ok(CommandResponse::success_with_data(
            serde_json::json!({ "path": engine_path_str })
        ));
    }
    
    // Production mode - try to find the engine relative to the executable
    // In a bundled Tauri app, the executable is in the app bundle
    // The engine should be in the same directory or a resources directory
    
    // First, try to find it relative to the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Try common locations in a Tauri bundle
            let mut possible_paths = vec![
                exe_dir.join("usi-engine"),
                exe_dir.join("resources").join("usi-engine"),
            ];
            
            // Add macOS Resources path if it exists
            if let Some(resources_path) = exe_dir.parent()
                .and_then(|p| p.parent())
                .map(|p| p.join("Resources").join("usi-engine")) {
                possible_paths.push(resources_path);
            }
            
            // On Windows, also try with .exe extension
            #[cfg(target_os = "windows")]
            {
                let mut windows_paths = Vec::new();
                for p in &possible_paths {
                    if let Some(parent) = p.parent() {
                        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                            windows_paths.push(parent.join(format!("{}.exe", stem)));
                        }
                    }
                }
                // Check Windows paths first
                if let Some(path) = windows_paths.iter().find(|p| p.exists()) {
                    log::info!("Built-in engine path: {}", path.display());
                    return Ok(CommandResponse::success_with_data(
                        serde_json::json!({ "path": path.display().to_string() })
                    ));
                }
            }
            
            if let Some(path) = possible_paths.iter().find(|p| p.exists()) {
                log::info!("Built-in engine path: {}", path.display());
                return Ok(CommandResponse::success_with_data(
                    serde_json::json!({ "path": path.display().to_string() })
                ));
            }
        }
    }
    
    // Fallback: try resource directory
    if let Ok(resource_dir) = app_handle.path().resource_dir() {
        let engine_path = resource_dir.join("usi-engine");
        #[cfg(target_os = "windows")]
        let engine_path = resource_dir.join("usi-engine.exe");
        
        if engine_path.exists() {
            log::info!("Built-in engine path: {}", engine_path.display());
            return Ok(CommandResponse::success_with_data(
                serde_json::json!({ "path": engine_path.display().to_string() })
            ));
        }
    }
    
    // Last resort: try workspace root (for development builds that are "release")
    if let Some(workspace_root) = find_workspace_root() {
        let engine_path = workspace_root.join("target/release/usi-engine");
        if engine_path.exists() {
            log::info!("Built-in engine path: {}", engine_path.display());
            return Ok(CommandResponse::success_with_data(
                serde_json::json!({ "path": engine_path.display().to_string() })
            ));
        }
    }
    
    // Could not find engine
    Ok(CommandResponse::error(
        "Engine binary not found in production bundle".to_string()
    ))
}

/// Add a new engine to the configuration
#[tauri::command]
pub async fn add_engine(
    name: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::info!("Command: add_engine - name: {}, path: {}", name, path);

    // Validate the engine
    let metadata = match engine_validator::validate_engine(&path).await {
        Ok(meta) => {
            log::info!("Engine validation successful: {}", meta.name);
            Some(meta)
        }
        Err(e) => {
            log::error!("Engine validation failed: {}", e);
            return Ok(CommandResponse::error(format!("Engine validation failed: {}", e)));
        }
    };

    // Create engine config
    let config = EngineConfig::new(name, path, metadata, false);
    let engine_id = config.id.clone();

    // Add to storage
    let mut storage = state.engine_storage.write().await;
    match storage.add_engine(config.clone()) {
        Ok(_) => {
            // Save to disk
            if let Err(e) = storage.save().await {
                log::error!("Failed to save engine storage: {}", e);
                return Ok(CommandResponse::error(format!("Failed to save configuration: {}", e)));
            }

            log::info!("Engine added successfully: {}", engine_id);
            Ok(CommandResponse::success_with_data(
                serde_json::to_value(&config).unwrap_or(serde_json::json!({}))
            ))
        }
        Err(e) => {
            log::error!("Failed to add engine: {}", e);
            Ok(CommandResponse::error(format!("Failed to add engine: {}", e)))
        }
    }
}

/// Remove an engine from the configuration
#[tauri::command]
pub async fn remove_engine(
    engine_id: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::info!("Command: remove_engine - engine_id: {}", engine_id);

    let mut storage = state.engine_storage.write().await;
    
    // Check if it's the built-in engine
    if let Some(engine) = storage.get_engine(&engine_id) {
        if engine.is_builtin {
            return Ok(CommandResponse::error("Cannot remove the built-in engine".to_string()));
        }
    }

    match storage.remove_engine(&engine_id) {
        Ok(_) => {
            // Save to disk
            if let Err(e) = storage.save().await {
                log::error!("Failed to save engine storage: {}", e);
                return Ok(CommandResponse::error(format!("Failed to save configuration: {}", e)));
            }

            log::info!("Engine removed successfully: {}", engine_id);
            Ok(CommandResponse::success())
        }
        Err(e) => {
            log::error!("Failed to remove engine: {}", e);
            Ok(CommandResponse::error(format!("Failed to remove engine: {}", e)))
        }
    }
}

/// Get all configured engines
#[tauri::command]
pub async fn get_engines(
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    let storage = state.engine_storage.read().await;
    let engines = storage.get_all_engines();
    
    Ok(CommandResponse::success_with_data(
        serde_json::to_value(engines).unwrap_or(serde_json::json!([]))
    ))
}

/// Validate an engine at a given path
#[tauri::command]
pub async fn validate_engine_path(
    path: String,
) -> Result<CommandResponse, String> {
    log::info!("Command: validate_engine_path - path: {}", path);

    match engine_validator::validate_engine(&path).await {
        Ok(metadata) => {
            log::info!("Engine validation successful: {}", metadata.name);
            Ok(CommandResponse::success_with_data(
                serde_json::to_value(&metadata).unwrap_or(serde_json::json!({}))
            ))
        }
        Err(e) => {
            log::error!("Engine validation failed: {}", e);
            Ok(CommandResponse::error(format!("Validation failed: {}", e)))
        }
    }
}

/// Register the built-in engine if not already present, or update the path if it's incorrect
#[tauri::command]
pub async fn register_builtin_engine(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::info!("Command: register_builtin_engine");

    // Get the correct built-in engine path first
    let path_response = get_builtin_engine_path(app_handle).await?;
    if !path_response.success {
        return Ok(path_response);
    }

    let engine_path = path_response
        .data
        .and_then(|d| d.get("path").and_then(|p| p.as_str().map(String::from)))
        .ok_or_else(|| "Failed to get engine path".to_string())?;

    let mut storage = state.engine_storage.write().await;

    // Check if already registered - if so, update path if it's different
    if let Some(builtin_engine) = storage.engines.iter_mut().find(|e| e.is_builtin) {
        let path_exists = std::path::Path::new(&builtin_engine.path).exists();
        let path_is_correct = builtin_engine.path == engine_path;
        
        if path_is_correct && path_exists {
            log::info!("Built-in engine already registered with correct path");
            return Ok(CommandResponse::success_with_data(
                serde_json::json!({ "already_registered": true })
            ));
        }
        
        // Path is incorrect or file doesn't exist - update it
        log::info!("Updating built-in engine path from '{}' to '{}'", builtin_engine.path, engine_path);
        builtin_engine.path = engine_path.clone();
        
        // Validate the new path and update metadata
        let metadata = match engine_validator::validate_engine(&engine_path).await {
            Ok(meta) => Some(meta),
            Err(e) => {
                log::warn!("Built-in engine validation failed: {}", e);
                None
            }
        };
        builtin_engine.metadata = metadata;
        
        // Save to disk
        if let Err(e) = storage.save().await {
            log::error!("Failed to save engine storage: {}", e);
            return Ok(CommandResponse::error(format!("Failed to save configuration: {}", e)));
        }
        
        log::info!("Built-in engine path updated successfully");
        return Ok(CommandResponse::success_with_data(
            serde_json::json!({ "updated": true, "path": engine_path })
        ));
    }

    // Validate the built-in engine (for new registration)
    let metadata = match engine_validator::validate_engine(&engine_path).await {
        Ok(meta) => Some(meta),
        Err(e) => {
            log::warn!("Built-in engine validation failed: {}", e);
            None
        }
    };

    // Create config for built-in engine
    let config = EngineConfig::new(
        "Built-in Engine".to_string(),
        engine_path,
        metadata,
        true,
    );

    // Add to storage
    match storage.add_engine(config.clone()) {
        Ok(_) => {
            // Save to disk
            if let Err(e) = storage.save().await {
                log::error!("Failed to save engine storage: {}", e);
                return Ok(CommandResponse::error(format!("Failed to save configuration: {}", e)));
            }

            log::info!("Built-in engine registered successfully");
            Ok(CommandResponse::success_with_data(
                serde_json::to_value(&config).unwrap_or(serde_json::json!({}))
            ))
        }
        Err(e) => {
            log::error!("Failed to register built-in engine: {}", e);
            Ok(CommandResponse::error(format!("Failed to register engine: {}", e)))
        }
    }
}

/// Perform health checks on all configured engines
#[tauri::command]
pub async fn health_check_engines(
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::info!("Command: health_check_engines");

    let storage = state.engine_storage.read().await;
    let engines = storage.get_all_engines();
    let mut results = Vec::new();

    for engine in engines {
        if !engine.enabled {
            results.push(serde_json::json!({
                "id": engine.id,
                "name": engine.name,
                "status": "disabled",
            }));
            continue;
        }

        log::info!("Health checking engine: {}", engine.name);
        match engine_validator::validate_engine(&engine.path).await {
            Ok(_) => {
                results.push(serde_json::json!({
                    "id": engine.id,
                    "name": engine.name,
                    "status": "healthy",
                }));
            }
            Err(e) => {
                log::warn!("Engine {} health check failed: {}", engine.name, e);
                results.push(serde_json::json!({
                    "id": engine.id,
                    "name": engine.name,
                    "status": "unhealthy",
                    "error": e.to_string(),
                }));
            }
        }
    }

    Ok(CommandResponse::success_with_data(
        serde_json::json!({ "results": results })
    ))
}

/// Start an engine-vs-engine match
#[tauri::command]
pub async fn start_engine_vs_engine(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    engine1_id: String,
    engine2_id: String,
    initial_sfen: Option<String>,
    time_per_move_ms: Option<u64>,
    max_moves: Option<usize>,
) -> Result<CommandResponse, String> {
    log::info!("Command: start_engine_vs_engine - {} vs {}", engine1_id, engine2_id);

    // Get engine configurations
    let storage = state.engine_storage.read().await;
    
    let engine1 = storage.get_engine(&engine1_id)
        .ok_or_else(|| "Engine 1 not found".to_string())?;
    let engine2 = storage.get_engine(&engine2_id)
        .ok_or_else(|| "Engine 2 not found".to_string())?;

    let config = EngineVsEngineConfig {
        engine1_id: engine1_id.clone(),
        engine1_path: engine1.path.clone(),
        engine1_name: engine1.name.clone(),
        engine2_id: engine2_id.clone(),
        engine2_path: engine2.path.clone(),
        engine2_name: engine2.name.clone(),
        initial_sfen,
        time_per_move_ms: time_per_move_ms.unwrap_or(5000),
        max_moves: max_moves.unwrap_or(200),
    };

    drop(storage);

    // Spawn the game loop in a background task
    let manager = EngineVsEngineManager::new(app_handle, config, state.engine_storage.clone());
    
    tokio::spawn(async move {
        if let Err(e) = manager.run_match().await {
            log::error!("Engine-vs-engine match error: {}", e);
        }
    });

    Ok(CommandResponse::success())
}

/// Save engine options
#[tauri::command]
pub async fn save_engine_options(
    engine_id: String,
    options: std::collections::HashMap<String, String>,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::info!("Command: save_engine_options - engine_id: {}, options: {:?}", engine_id, options);

    let mut storage = state.engine_storage.write().await;
    
    match storage.save_engine_options(&engine_id, options) {
        Ok(_) => {
            // Save to disk
            if let Err(e) = storage.save().await {
                log::error!("Failed to save engine storage: {}", e);
                return Ok(CommandResponse::error(format!("Failed to save options: {}", e)));
            }
            
            log::info!("Engine options saved successfully for engine: {}", engine_id);
            Ok(CommandResponse::success())
        }
        Err(e) => {
            log::error!("Failed to save engine options: {}", e);
            Ok(CommandResponse::error(format!("Failed to save options: {}", e)))
        }
    }
}

/// Get saved engine options
#[tauri::command]
pub async fn get_engine_options(
    engine_id: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::info!("Command: get_engine_options - engine_id: {}", engine_id);

    let storage = state.engine_storage.read().await;
    
    match storage.get_engine_options(&engine_id) {
        Some(options) => {
            log::info!("Retrieved {} saved options for engine: {}", options.len(), engine_id);
            Ok(CommandResponse::success_with_data(serde_json::to_value(options).unwrap()))
        }
        None => {
            log::info!("No saved options found for engine: {}", engine_id);
            Ok(CommandResponse::success_with_data(serde_json::Value::Object(serde_json::Map::new())))
        }
    }
}

/// Clone an engine with a new display name
#[tauri::command]
pub async fn clone_engine(
    engine_id: String,
    new_display_name: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::info!("Command: clone_engine - engine_id: {}, new_display_name: {}", engine_id, new_display_name);

    let mut storage = state.engine_storage.write().await;
    
    match storage.clone_engine(&engine_id, new_display_name) {
        Ok(new_engine_id) => {
            // Save to disk
            if let Err(e) = storage.save().await {
                log::error!("Failed to save engine storage: {}", e);
                return Ok(CommandResponse::error(format!("Failed to save cloned engine: {}", e)));
            }
            
            log::info!("Engine cloned successfully: {} -> {}", engine_id, new_engine_id);
            Ok(CommandResponse::success_with_data(serde_json::json!({ "new_engine_id": new_engine_id })))
        }
        Err(e) => {
            log::error!("Failed to clone engine: {}", e);
            Ok(CommandResponse::error(format!("Failed to clone engine: {}", e)))
        }
    }
}

/// Update engine display name
#[tauri::command]
pub async fn update_engine_display_name(
    engine_id: String,
    new_display_name: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::info!("Command: update_engine_display_name - engine_id: {}, new_display_name: {}", engine_id, new_display_name);

    let mut storage = state.engine_storage.write().await;
    
    match storage.update_display_name(&engine_id, new_display_name) {
        Ok(_) => {
            // Save to disk
            if let Err(e) = storage.save().await {
                log::error!("Failed to save engine storage: {}", e);
                return Ok(CommandResponse::error(format!("Failed to save display name: {}", e)));
            }
            
            log::info!("Engine display name updated successfully: {}", engine_id);
            Ok(CommandResponse::success())
        }
        Err(e) => {
            log::error!("Failed to update display name: {}", e);
            Ok(CommandResponse::error(format!("Failed to update display name: {}", e)))
        }
    }
}

/// Set an engine as favorite
#[tauri::command]
pub async fn set_favorite_engine(
    engine_id: String,
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::info!("Command: set_favorite_engine - engine_id: {}", engine_id);

    let mut storage = state.engine_storage.write().await;
    
    match storage.set_favorite_engine(&engine_id) {
        Ok(_) => {
            // Save to disk
            if let Err(e) = storage.save().await {
                log::error!("Failed to save engine storage: {}", e);
                return Ok(CommandResponse::error(format!("Failed to save favorite status: {}", e)));
            }
            
            log::info!("Engine set as favorite successfully: {}", engine_id);
            Ok(CommandResponse::success())
        }
        Err(e) => {
            log::error!("Failed to set favorite engine: {}", e);
            Ok(CommandResponse::error(format!("Failed to set favorite engine: {}", e)))
        }
    }
}

