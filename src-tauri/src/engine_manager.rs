use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::timeout;

/// Represents the status of a USI engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EngineStatus {
    Starting,
    Ready,
    Thinking,
    Error,
    Stopped,
}

/// Represents a USI engine instance
#[derive(Debug)]
pub struct EngineInstance {
    pub id: String,
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub path: String,
    pub status: EngineStatus,
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    #[allow(dead_code)]
    command_tx: mpsc::Sender<String>,
    stop_tx: mpsc::Sender<()>,
}

impl EngineInstance {
    /// Create a new engine instance (doesn't start the process yet)
    pub fn new(id: String, name: String, path: String) -> Self {
        let (command_tx, _command_rx) = mpsc::channel(100);
        let (stop_tx, _stop_rx) = mpsc::channel(1);
        
        Self {
            id,
            name,
            path,
            status: EngineStatus::Stopped,
            process: None,
            stdin: None,
            command_tx,
            stop_tx,
        }
    }

    /// Send a USI command to the engine
    pub async fn send_command(&mut self, command: &str) -> Result<()> {
        if let Some(stdin) = &mut self.stdin {
            stdin.write_all(command.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;
            log::debug!("Sent command to engine {}: {}", self.id, command);
            Ok(())
        } else {
            Err(anyhow!("Engine stdin not available"))
        }
    }

    /// Stop the engine process
    pub async fn stop(&mut self) -> Result<()> {
        log::info!("Stopping engine: {}", self.id);
        
        // Try to send quit command gracefully
        if let Err(e) = self.send_command("quit").await {
            log::warn!("Failed to send quit command to engine {}: {}", self.id, e);
        }

        // Signal the output reader task to stop
        let _ = self.stop_tx.send(()).await;

        // Kill the process if it doesn't stop gracefully
        if let Some(process) = &mut self.process {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let _ = process.kill().await;
        }

        self.status = EngineStatus::Stopped;
        self.process = None;
        self.stdin = None;

        Ok(())
    }
}

/// Manages all USI engine instances
pub struct EngineManager {
    engines: Arc<RwLock<HashMap<String, Arc<Mutex<EngineInstance>>>>>,
    app_handle: AppHandle,
}

impl EngineManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            engines: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
        }
    }

    /// Spawn a new engine process
    pub async fn spawn_engine(
        &self,
        id: String,
        name: String,
        path: String,
    ) -> Result<String> {
        log::info!("Spawning engine: {} at path: {}", name, path);

        // Create engine instance
        let mut engine = EngineInstance::new(id.clone(), name.clone(), path.clone());
        engine.status = EngineStatus::Starting;

        // Determine working directory - use the engine's directory
        // This is critical for engines like Apery that need access to data files
        let working_dir = std::path::Path::new(&path)
            .parent()
            .map(|p| p.to_path_buf());
        
        log::info!("Engine working directory: {:?}", working_dir);
        
        // Spawn the process
        let mut command = Command::new(&path);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        
        // Set working directory if we have one
        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }
        
        let mut child = command.spawn()
            .map_err(|e| anyhow!("Failed to spawn engine process: {}", e))?;

        log::info!("Engine process spawned, PID: {:?}", child.id());

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("Failed to get stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("Failed to get stdout"))?;
        let stderr = child.stderr.take().ok_or_else(|| anyhow!("Failed to get stderr"))?;

        engine.process = Some(child);
        engine.stdin = Some(stdin);

        let engine_arc = Arc::new(Mutex::new(engine));

        // Store the engine
        {
            let mut engines = self.engines.write().await;
            engines.insert(id.clone(), engine_arc.clone());
        }

        // Spawn stdout reader task
        self.spawn_output_reader(id.clone(), stdout).await;

        // Spawn stderr reader task
        self.spawn_error_reader(id.clone(), stderr).await;

        // Spawn watchdog task
        self.spawn_watchdog(id.clone()).await;

        // Give the engine process a moment to start up before we try to communicate
        // This prevents race conditions where we try to write to stdin before the engine is ready
        tokio::time::sleep(Duration::from_millis(100)).await;

        log::info!("Engine {} spawned successfully", id);
        Ok(id)
    }

    /// Spawn a task to read engine stdout and emit events
    async fn spawn_output_reader(&self, engine_id: String, stdout: ChildStdout) {
        let app_handle = self.app_handle.clone();
        let engines = self.engines.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            let mut line_count = 0;
            while let Ok(Some(line)) = lines.next_line().await {
                line_count += 1;
                log::debug!("Engine {} output: {}", engine_id, line);

                // Update engine status based on output
                if line.contains("usiok") {
                    if let Some(engine) = engines.read().await.get(&engine_id) {
                        engine.lock().await.status = EngineStatus::Ready;
                    }
                } else if line.contains("readyok") {
                    if let Some(engine) = engines.read().await.get(&engine_id) {
                        engine.lock().await.status = EngineStatus::Ready;
                    }
                } else if line.starts_with("bestmove") {
                    if let Some(engine) = engines.read().await.get(&engine_id) {
                        engine.lock().await.status = EngineStatus::Ready;
                    }
                }

                // Emit event to frontend
                let event_name = format!("usi-message::{}", engine_id);
                if let Err(e) = app_handle.emit(&event_name, &line) {
                    log::error!("Failed to emit USI message event: {}", e);
                }
            }

            log::warn!("Engine {} stdout reader task ended after {} lines", engine_id, line_count);
        });
    }

    /// Spawn a task to read engine stderr and emit error events
    async fn spawn_error_reader(&self, engine_id: String, stderr: tokio::process::ChildStderr) {
        let app_handle = self.app_handle.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            let mut line_count = 0;
            while let Ok(Some(line)) = lines.next_line().await {
                line_count += 1;
                log::warn!("Engine {} stderr: {}", engine_id, line);

                // Emit error event to frontend
                let event_name = format!("usi-error::{}", engine_id);
                if let Err(e) = app_handle.emit(&event_name, &line) {
                    log::error!("Failed to emit USI error event: {}", e);
                }
            }

            log::warn!("Engine {} stderr reader task ended after {} lines", engine_id, line_count);
        });
    }

    /// Spawn a watchdog task to detect hangs and crashes
    async fn spawn_watchdog(&self, engine_id: String) {
        let engines = self.engines.clone();
        let app_handle = self.app_handle.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;

                let engines_lock = engines.read().await;
                if let Some(engine) = engines_lock.get(&engine_id) {
                    let engine_lock = engine.lock().await;
                    
                    // Check if process is still alive
                    if let Some(process) = &engine_lock.process {
                        match process.id() {
                            Some(_) => {
                                // Process is alive, continue
                            }
                            None => {
                                log::error!("Engine {} process died", engine_id);
                                drop(engine_lock);
                                drop(engines_lock);
                                
                                // Update status and emit event
                                if let Some(engine) = engines.read().await.get(&engine_id) {
                                    engine.lock().await.status = EngineStatus::Error;
                                }
                                
                                let event_name = format!("usi-error::{}", engine_id);
                                let _ = app_handle.emit(&event_name, "Engine process died");
                                break;
                            }
                        }
                    } else {
                        // Engine stopped, exit watchdog
                        break;
                    }
                } else {
                    // Engine removed from manager, exit watchdog
                    break;
                }
            }

            log::info!("Engine {} watchdog task ended", engine_id);
        });
    }

    /// Send a USI command to a specific engine
    /// Supports both runtime IDs (full ID) and config IDs (prefix match)
    pub async fn send_command(&self, engine_id: &str, command: &str) -> Result<()> {
        let engines = self.engines.read().await;
        
        // First try exact match (runtime ID)
        let engine = if let Some(engine) = engines.get(engine_id) {
            Some(engine.clone())
        } else {
            // Try prefix match (config ID) - find engine whose ID starts with the given ID
            engines
                .iter()
                .find(|(id, _)| id.starts_with(engine_id))
                .map(|(_, engine)| engine.clone())
        }
        .ok_or_else(|| anyhow!("Engine not found: {}", engine_id))?;

        let mut engine_lock = engine.lock().await;
        engine_lock.send_command(command).await
    }

    /// Send a USI command with timeout
    pub async fn send_command_with_timeout(
        &self,
        engine_id: &str,
        command: &str,
        timeout_duration: Duration,
    ) -> Result<()> {
        timeout(timeout_duration, self.send_command(engine_id, command))
            .await
            .map_err(|_| anyhow!("Command timeout"))?
    }


    /// Initialize an engine with temporary options (for one-time game use)
    /// If temp_options is Some, use those; otherwise fall back to saved options
    pub async fn initialize_engine_with_temp_options(
        &self, 
        engine_id: &str, 
        engine_storage: &tokio::sync::RwLock<crate::engine_storage::EngineStorage>,
        temp_options: Option<&std::collections::HashMap<String, String>>
    ) -> Result<()> {
        log::info!("Initializing engine with {} options: {}", 
            if temp_options.is_some() { "temporary" } else { "saved" }, 
            engine_id
        );

        // Send usi command
        log::info!("Sending 'usi' command to engine: {}", engine_id);
        self.send_command_with_timeout(engine_id, "usi", Duration::from_secs(5))
            .await?;

        // Wait for usiok response by polling engine status
        log::info!("Waiting for usiok from engine: {}", engine_id);
        let start = tokio::time::Instant::now();
        loop {
            if start.elapsed() > Duration::from_secs(10) {
                return Err(anyhow!("Timeout waiting for usiok"));
            }
            
            let engines = self.engines.read().await;
            // Try exact match first, then prefix match
            let engine = engines.get(engine_id)
                .or_else(|| engines.iter().find(|(id, _)| id.starts_with(engine_id)).map(|(_, e)| e));
            
            if let Some(engine) = engine {
                let status = engine.lock().await.status.clone();
                if matches!(status, EngineStatus::Ready) {
                    log::info!("Received usiok from engine: {}", engine_id);
                    break;
                }
            } else {
                return Err(anyhow!("Engine not found"));
            }
            
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Send options (temporary or saved)
        if let Some(options) = temp_options {
            // Use temporary options
            if !options.is_empty() {
                log::info!("Sending {} temporary options to engine: {}", options.len(), engine_id);
                for (option_name, option_value) in options {
                    let option_command = format!("setoption name {} value {}", option_name, option_value);
                    log::debug!("Sending option command: {}", option_command);
                    if let Err(e) = self.send_command_with_timeout(engine_id, &option_command, Duration::from_secs(2)).await {
                        log::warn!("Failed to send option '{}' to engine {}: {}", option_name, engine_id, e);
                    }
                }
            }
        } else {
            // Use saved options from storage
            let storage = engine_storage.read().await;
            if let Some(options) = storage.get_engine_options(engine_id) {
                if !options.is_empty() {
                    log::info!("Sending {} saved options to engine: {}", options.len(), engine_id);
                    for (option_name, option_value) in options {
                        let option_command = format!("setoption name {} value {}", option_name, option_value);
                        log::debug!("Sending option command: {}", option_command);
                        if let Err(e) = self.send_command_with_timeout(engine_id, &option_command, Duration::from_secs(2)).await {
                            log::warn!("Failed to send option '{}' to engine {}: {}", option_name, engine_id, e);
                        }
                    }
                }
            }
            drop(storage);
        }

        // Send isready command
        log::info!("Sending 'isready' command to engine: {}", engine_id);
        self.send_command_with_timeout(engine_id, "isready", Duration::from_secs(5))
            .await?;

        // Wait for readyok response by polling engine status
        log::info!("Waiting for readyok from engine: {}", engine_id);
        let start = tokio::time::Instant::now();
        loop {
            if start.elapsed() > Duration::from_secs(10) {
                return Err(anyhow!("Timeout waiting for readyok"));
            }
            
            let engines = self.engines.read().await;
            // Try exact match first, then prefix match
            let engine = engines.get(engine_id)
                .or_else(|| engines.iter().find(|(id, _)| id.starts_with(engine_id)).map(|(_, e)| e));
            
            if let Some(engine) = engine {
                let status = engine.lock().await.status.clone();
                if matches!(status, EngineStatus::Ready) {
                    log::info!("Received readyok from engine: {}", engine_id);
                    break;
                }
            } else {
                return Err(anyhow!("Engine not found"));
            }
            
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        log::info!("Engine initialization complete: {}", engine_id);
        Ok(())
    }


    /// Stop a specific engine
    /// Supports both runtime IDs (full ID) and config IDs (prefix match)
    pub async fn stop_engine(&self, engine_id: &str) -> Result<()> {
        let engines = self.engines.read().await;
        
        // First try exact match (runtime ID)
        let (actual_id, engine) = if let Some(engine) = engines.get(engine_id) {
            (engine_id.to_string(), Some(engine.clone()))
        } else {
            // Try prefix match (config ID) - find engine whose ID starts with the given ID
            engines
                .iter()
                .find(|(id, _)| id.starts_with(engine_id))
                .map(|(id, engine)| (id.clone(), Some(engine.clone())))
                .unwrap_or_else(|| (engine_id.to_string(), None))
        };
        
        let engine = engine.ok_or_else(|| anyhow!("Engine not found: {}", engine_id))?;

        let mut engine_lock = engine.lock().await;
        engine_lock.stop().await?;

        drop(engine_lock);
        drop(engines);

        // Remove from manager using the actual runtime ID
        self.engines.write().await.remove(&actual_id);

        Ok(())
    }

    /// Get engine status
    /// Supports both runtime IDs (full ID) and config IDs (prefix match)
    pub async fn get_engine_status(&self, engine_id: &str) -> Option<EngineStatus> {
        let engines = self.engines.read().await;
        
        // First try exact match (runtime ID)
        let engine = if let Some(engine) = engines.get(engine_id) {
            Some(engine.clone())
        } else {
            // Try prefix match (config ID) - find engine whose ID starts with the given ID
            engines
                .iter()
                .find(|(id, _)| id.starts_with(engine_id))
                .map(|(_, engine)| engine.clone())
        };
        
        engine.map(|engine| {
            let engine_lock = futures::executor::block_on(engine.lock());
            engine_lock.status.clone()
        })
    }

    /// Get list of all engine IDs
    pub async fn list_engines(&self) -> Vec<String> {
        self.engines.read().await.keys().cloned().collect()
    }

    /// Stop all engines
    pub async fn stop_all_engines(&self) -> Result<()> {
        let engine_ids: Vec<String> = self.list_engines().await;

        for engine_id in engine_ids {
            if let Err(e) = self.stop_engine(&engine_id).await {
                log::error!("Failed to stop engine {}: {}", engine_id, e);
            }
        }

        Ok(())
    }
}

