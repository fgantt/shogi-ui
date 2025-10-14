use crate::engine_manager::EngineManager;
use std::sync::Arc;

/// Application state that is shared across the Tauri app
pub struct AppState {
    pub engine_manager: Arc<EngineManager>,
}

impl AppState {
    pub fn new(engine_manager: EngineManager) -> Self {
        Self {
            engine_manager: Arc::new(engine_manager),
        }
    }
}

