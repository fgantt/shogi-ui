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

        // Spawn the process
        let mut child = Command::new(&path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn engine process: {}", e))?;

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

            while let Ok(Some(line)) = lines.next_line().await {
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

            log::info!("Engine {} stdout reader task ended", engine_id);
        });
    }

    /// Spawn a task to read engine stderr and emit error events
    async fn spawn_error_reader(&self, engine_id: String, stderr: tokio::process::ChildStderr) {
        let app_handle = self.app_handle.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                log::warn!("Engine {} stderr: {}", engine_id, line);

                // Emit error event to frontend
                let event_name = format!("usi-error::{}", engine_id);
                if let Err(e) = app_handle.emit(&event_name, &line) {
                    log::error!("Failed to emit USI error event: {}", e);
                }
            }

            log::info!("Engine {} stderr reader task ended", engine_id);
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
    pub async fn send_command(&self, engine_id: &str, command: &str) -> Result<()> {
        let engines = self.engines.read().await;
        let engine = engines
            .get(engine_id)
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

    /// Initialize an engine with the USI protocol
    pub async fn initialize_engine(&self, engine_id: &str) -> Result<()> {
        log::info!("Initializing engine: {}", engine_id);

        // Send usi command with 5 second timeout
        self.send_command_with_timeout(engine_id, "usi", Duration::from_secs(5))
            .await?;

        // Wait a bit for usiok response
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Send isready command
        self.send_command_with_timeout(engine_id, "isready", Duration::from_secs(5))
            .await?;

        Ok(())
    }

    /// Stop a specific engine
    pub async fn stop_engine(&self, engine_id: &str) -> Result<()> {
        let engines = self.engines.read().await;
        let engine = engines
            .get(engine_id)
            .ok_or_else(|| anyhow!("Engine not found: {}", engine_id))?;

        let mut engine_lock = engine.lock().await;
        engine_lock.stop().await?;

        drop(engine_lock);
        drop(engines);

        // Remove from manager
        self.engines.write().await.remove(engine_id);

        Ok(())
    }

    /// Get engine status
    pub async fn get_engine_status(&self, engine_id: &str) -> Option<EngineStatus> {
        let engines = self.engines.read().await;
        engines.get(engine_id).map(|engine| {
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

