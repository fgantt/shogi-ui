use crate::engine_validator::EngineMetadata;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Configuration for a stored engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub id: String,
    pub name: String,
    pub path: String,
    pub metadata: Option<EngineMetadata>,
    pub is_builtin: bool,
    pub enabled: bool,
    pub last_used: Option<String>,
    pub created_at: String,
}

impl EngineConfig {
    pub fn new(name: String, path: String, metadata: Option<EngineMetadata>, is_builtin: bool) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            path,
            metadata,
            is_builtin,
            enabled: true,
            last_used: None,
            created_at: now,
        }
    }
}

/// Storage container for all engine configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStorage {
    pub version: String,
    pub engines: Vec<EngineConfig>,
}

impl Default for EngineStorage {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            engines: Vec::new(),
        }
    }
}

impl EngineStorage {
    /// Get the platform-appropriate storage path
    pub fn get_storage_path() -> Result<PathBuf> {
        let config_dir = if cfg!(target_os = "windows") {
            // Windows: %APPDATA%\shogi-vibe
            std::env::var("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("shogi-vibe")
        } else {
            // Linux/macOS: ~/.config/shogi-vibe
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("shogi-vibe")
        };

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&config_dir)?;

        Ok(config_dir.join("engines.json"))
    }

    /// Load engine storage from disk
    pub async fn load() -> Result<Self> {
        let path = Self::get_storage_path()?;
        
        if !path.exists() {
            log::info!("Engine storage file not found, creating new storage");
            return Ok(Self::default());
        }

        log::info!("Loading engine storage from: {}", path.display());
        let contents = tokio::fs::read_to_string(&path).await?;
        let storage: Self = serde_json::from_str(&contents)?;
        
        log::info!("Loaded {} engines from storage", storage.engines.len());
        Ok(storage)
    }

    /// Save engine storage to disk
    pub async fn save(&self) -> Result<()> {
        let path = Self::get_storage_path()?;
        log::info!("Saving engine storage to: {}", path.display());
        
        let contents = serde_json::to_string_pretty(self)?;
        tokio::fs::write(&path, contents).await?;
        
        log::info!("Saved {} engines to storage", self.engines.len());
        Ok(())
    }

    /// Add a new engine configuration
    pub fn add_engine(&mut self, config: EngineConfig) -> Result<String> {
        // Check if an engine with the same path already exists
        if self.engines.iter().any(|e| e.path == config.path) {
            return Err(anyhow!("An engine with this path is already configured"));
        }

        let id = config.id.clone();
        self.engines.push(config);
        Ok(id)
    }

    /// Remove an engine by ID
    pub fn remove_engine(&mut self, engine_id: &str) -> Result<()> {
        let initial_len = self.engines.len();
        self.engines.retain(|e| e.id != engine_id);
        
        if self.engines.len() == initial_len {
            return Err(anyhow!("Engine not found: {}", engine_id));
        }
        
        Ok(())
    }

    /// Get an engine by ID
    pub fn get_engine(&self, engine_id: &str) -> Option<&EngineConfig> {
        self.engines.iter().find(|e| e.id == engine_id)
    }

    /// Get a mutable reference to an engine by ID
    #[allow(dead_code)]
    pub fn get_engine_mut(&mut self, engine_id: &str) -> Option<&mut EngineConfig> {
        self.engines.iter_mut().find(|e| e.id == engine_id)
    }

    /// Update last used timestamp for an engine
    #[allow(dead_code)]
    pub fn update_last_used(&mut self, engine_id: &str) -> Result<()> {
        let engine = self
            .get_engine_mut(engine_id)
            .ok_or_else(|| anyhow!("Engine not found"))?;
        
        engine.last_used = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    /// Check if the built-in engine is registered
    pub fn has_builtin_engine(&self) -> bool {
        self.engines.iter().any(|e| e.is_builtin)
    }

    /// Get all engine configurations
    pub fn get_all_engines(&self) -> &[EngineConfig] {
        &self.engines
    }

    /// Enable or disable an engine
    #[allow(dead_code)]
    pub fn set_engine_enabled(&mut self, engine_id: &str, enabled: bool) -> Result<()> {
        let engine = self
            .get_engine_mut(engine_id)
            .ok_or_else(|| anyhow!("Engine not found"))?;
        
        engine.enabled = enabled;
        Ok(())
    }
}

