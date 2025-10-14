use crate::engine_manager::EngineStatus;
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
    state: State<'_, AppState>,
) -> Result<CommandResponse, String> {
    log::info!("Command: spawn_engine - id: {}, name: {}, path: {}", engine_id, name, path);

    let manager = &state.engine_manager;
    
    match manager.spawn_engine(engine_id.clone(), name, path).await {
        Ok(_) => {
            // Initialize the engine with USI protocol
            if let Err(e) = manager.initialize_engine(&engine_id).await {
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

/// Get the path to the bundled built-in engine
#[tauri::command]
pub async fn get_builtin_engine_path(
    app_handle: tauri::AppHandle,
) -> Result<CommandResponse, String> {
    use tauri::Manager;

    // For development, use the release build from the project
    // In production, this would be bundled with the app
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    // For now, return the path to the development build
    // This will need to be updated for production deployment
    let engine_path = if cfg!(debug_assertions) {
        // Development mode - use the target/release build
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            .and_then(|dir| {
                // Try to find the engine in the workspace
                let workspace_root = dir.parent()?.parent()?.parent()?.parent()?;
                Some(workspace_root.join("target/release/shogi-engine"))
            })
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "../target/release/shogi-engine".to_string())
    } else {
        // Production mode - engine should be bundled
        app_dir.join("shogi-engine").display().to_string()
    };

    log::info!("Built-in engine path: {}", engine_path);
    
    // Check if the engine exists
    if std::path::Path::new(&engine_path).exists() {
        Ok(CommandResponse::success_with_data(
            serde_json::json!({ "path": engine_path })
        ))
    } else {
        let msg = format!("Engine not found at path: {}", engine_path);
        log::warn!("{}", msg);
        Ok(CommandResponse::error(msg))
    }
}

