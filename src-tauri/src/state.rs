use crate::engine_manager::EngineManager;
use crate::engine_storage::EngineStorage;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Application state that is shared across the Tauri app
pub struct AppState {
    pub engine_manager: Arc<EngineManager>,
    pub engine_storage: Arc<RwLock<EngineStorage>>,
}

impl AppState {
    pub fn new(engine_manager: EngineManager, engine_storage: EngineStorage) -> Self {
        Self {
            engine_manager: Arc::new(engine_manager),
            engine_storage: Arc::new(RwLock::new(engine_storage)),
        }
    }
}

