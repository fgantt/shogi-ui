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
    #[serde(default = "default_display_name")]
    pub display_name: String,
    pub path: String,
    pub metadata: Option<EngineMetadata>,
    pub is_builtin: bool,
    pub enabled: bool,
    pub last_used: Option<String>,
    pub created_at: String,
    pub saved_options: Option<std::collections::HashMap<String, String>>,
}

fn default_display_name() -> String {
    String::new()
}

impl EngineConfig {
    pub fn new(name: String, path: String, metadata: Option<EngineMetadata>, is_builtin: bool) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.clone(),
            display_name: name,
            path,
            metadata,
            is_builtin,
            enabled: true,
            last_used: None,
            created_at: now,
            saved_options: None,
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
        let mut storage: Self = serde_json::from_str(&contents)?;
        
        // Migration: Set display_name to name if it's empty (for backwards compatibility)
        let mut needs_migration = false;
        for engine in &mut storage.engines {
            if engine.display_name.is_empty() {
                log::info!("Migrating engine '{}': setting display_name", engine.name);
                engine.display_name = engine.name.clone();
                needs_migration = true;
            }
        }
        
        // Save the migrated storage back to disk
        if needs_migration {
            log::info!("Saving migrated engine storage");
            storage.save().await?;
        }
        
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

    /// Save engine options
    pub fn save_engine_options(&mut self, engine_id: &str, options: std::collections::HashMap<String, String>) -> Result<()> {
        let engine = self
            .get_engine_mut(engine_id)
            .ok_or_else(|| anyhow!("Engine not found"))?;
        
        engine.saved_options = Some(options);
        Ok(())
    }

    /// Get saved engine options
    pub fn get_engine_options(&self, engine_id: &str) -> Option<&std::collections::HashMap<String, String>> {
        self.get_engine(engine_id)?.saved_options.as_ref()
    }

    /// Clone an engine with a new display name
    pub fn clone_engine(&mut self, engine_id: &str, new_display_name: String) -> Result<String> {
        let source_engine = self.get_engine(engine_id)
            .ok_or_else(|| anyhow!("Source engine not found: {}", engine_id))?
            .clone();

        let mut cloned_engine = source_engine;
        cloned_engine.id = Uuid::new_v4().to_string();
        cloned_engine.display_name = new_display_name;
        cloned_engine.is_builtin = false; // Cloned engines are never built-in
        cloned_engine.created_at = chrono::Utc::now().to_rfc3339();
        cloned_engine.last_used = None;

        let new_id = cloned_engine.id.clone();
        self.engines.push(cloned_engine);
        Ok(new_id)
    }

    /// Update display name for an engine
    pub fn update_display_name(&mut self, engine_id: &str, new_display_name: String) -> Result<()> {
        let engine = self
            .get_engine_mut(engine_id)
            .ok_or_else(|| anyhow!("Engine not found"))?;
        
        engine.display_name = new_display_name;
        Ok(())
    }
}

