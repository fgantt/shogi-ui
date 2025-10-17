/**
 * Engine vs Engine gameplay manager
 * Manages automated games between two engines with spectator mode
 */

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineVsEngineState {
    pub move_number: usize,
    pub current_player: String, // "black" or "white"
    pub position_sfen: String,
    pub last_move: Option<String>,
    pub move_history: Vec<String>,
    pub game_over: bool,
    pub winner: Option<String>,
    pub game_result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineVsEngineConfig {
    pub engine1_id: String,
    pub engine1_path: String,
    pub engine1_name: String,
    pub engine2_id: String,
    pub engine2_path: String,
    pub engine2_name: String,
    pub initial_sfen: Option<String>,
    pub time_per_move_ms: u64,
    pub max_moves: usize,
}

pub struct EngineVsEngineManager {
    app_handle: AppHandle,
    config: EngineVsEngineConfig,
    state: Arc<Mutex<EngineVsEngineState>>,
    engine1: Option<Child>,
    engine2: Option<Child>,
    engine_storage: Arc<tokio::sync::RwLock<crate::engine_storage::EngineStorage>>,
}

impl EngineVsEngineManager {
    pub fn new(app_handle: AppHandle, config: EngineVsEngineConfig, engine_storage: Arc<tokio::sync::RwLock<crate::engine_storage::EngineStorage>>) -> Self {
        let initial_sfen = config.initial_sfen.clone()
            .unwrap_or_else(|| "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string());

        let state = EngineVsEngineState {
            move_number: 1,
            current_player: "black".to_string(),
            position_sfen: initial_sfen,
            last_move: None,
            move_history: Vec::new(),
            game_over: false,
            winner: None,
            game_result: None,
        };

        Self {
            app_handle,
            config,
            state: Arc::new(Mutex::new(state)),
            engine1: None,
            engine2: None,
            engine_storage,
        }
    }

    /// Spawn both engines
    async fn spawn_engines(&mut self) -> Result<()> {
        log::info!("Spawning engines for engine-vs-engine match");
        log::info!("Engine 1 path: {}", self.config.engine1_path);
        log::info!("Engine 2 path: {}", self.config.engine2_path);

        // Spawn engine 1
        // Set working directory to the engine's directory so it can find its files
        let engine1_dir = std::path::Path::new(&self.config.engine1_path)
            .parent()
            .ok_or_else(|| anyhow!("Invalid engine 1 path"))?;
        
        let engine1 = Command::new(&self.config.engine1_path)
            .current_dir(engine1_dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn engine 1: {}", e))?;

        log::info!("Engine 1 spawned successfully with working dir: {:?}", engine1_dir);
        self.engine1 = Some(engine1);

        // Spawn engine 2
        let engine2_dir = std::path::Path::new(&self.config.engine2_path)
            .parent()
            .ok_or_else(|| anyhow!("Invalid engine 2 path"))?;
            
        let engine2 = Command::new(&self.config.engine2_path)
            .current_dir(engine2_dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn engine 2: {}", e))?;

        log::info!("Engine 2 spawned successfully");
        self.engine2 = Some(engine2);

        Ok(())
    }

    /// Initialize an engine with USI protocol and send saved options
    async fn initialize_engine_with_options(
        stdin: &mut tokio::process::ChildStdin,
        stdout: &mut tokio::process::ChildStdout,
        engine_id: &str,
        engine_storage: &tokio::sync::RwLock<crate::engine_storage::EngineStorage>,
    ) -> Result<()> {
        use tokio::io::AsyncBufReadExt;
        
        log::info!("Initializing engine with USI protocol");
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        
        // Send usi command
        log::info!("Sending 'usi' command");
        stdin.write_all(b"usi\n").await?;
        stdin.flush().await?;
        log::info!("'usi' command sent, waiting for response...");

        // Wait for usiok
        let mut found_usiok = false;
        let start = tokio::time::Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            line.clear();
            
            // Use a short timeout for each read to allow checking elapsed time
            match timeout(Duration::from_millis(100), reader.read_line(&mut line)).await {
                Ok(Ok(0)) => return Err(anyhow!("Engine closed connection")),
                Ok(Ok(_)) => {
                    let trimmed = line.trim();
                    log::debug!("Engine init response: {}", trimmed);
                    if trimmed == "usiok" {
                        found_usiok = true;
                        break;
                    }
                }
                Ok(Err(e)) => return Err(anyhow!("Failed to read from engine: {}", e)),
                Err(_) => continue, // Timeout, try again
            }
        }
        
        if !found_usiok {
            log::error!("Timeout waiting for usiok - no response from engine");
            return Err(anyhow!("Timeout waiting for usiok"));
        }

        log::info!("Received usiok, sending saved options");

        // Send saved options if any
        let storage = engine_storage.read().await;
        if let Some(options) = storage.get_engine_options(engine_id) {
            if !options.is_empty() {
                log::info!("Sending {} saved options to engine: {}", options.len(), engine_id);
                for (option_name, option_value) in options {
                    let option_command = format!("setoption name {} value {}\n", option_name, option_value);
                    log::debug!("Sending option command: {}", option_command.trim());
                    if let Err(e) = stdin.write_all(option_command.as_bytes()).await {
                        log::warn!("Failed to send option '{}' to engine {}: {}", option_name, engine_id, e);
                        // Continue with other options even if one fails
                    }
                }
                stdin.flush().await?;
            }
        }
        drop(storage);

        log::info!("Sending 'isready' command");
        // Send isready
        stdin.write_all(b"isready\n").await?;
        stdin.flush().await?;
        log::info!("'isready' command sent, waiting for response...");

        // Wait for readyok
        let mut found_readyok = false;
        let start = tokio::time::Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            line.clear();
            
            match timeout(Duration::from_millis(100), reader.read_line(&mut line)).await {
                Ok(Ok(0)) => return Err(anyhow!("Engine closed connection")),
                Ok(Ok(_)) => {
                    let trimmed = line.trim();
                    log::debug!("Engine ready response: {}", trimmed);
                    if trimmed == "readyok" {
                        found_readyok = true;
                        break;
                    }
                }
                Ok(Err(e)) => return Err(anyhow!("Failed to read from engine: {}", e)),
                Err(_) => continue, // Timeout, try again
            }
        }
        
        if !found_readyok {
            log::error!("Timeout waiting for readyok - no response from engine");
            return Err(anyhow!("Timeout waiting for readyok"));
        }

        log::info!("Received readyok, engine initialization complete");
        Ok(())
    }

    /// Request a move from an engine
    async fn request_move(
        stdin: &mut tokio::process::ChildStdin,
        stdout: &mut tokio::process::ChildStdout,
        position_sfen: &str,
        moves: &[String],
        time_ms: u64,
    ) -> Result<String> {
        use tokio::io::AsyncBufReadExt;
        
        // Build position command
        let pos_cmd = if moves.is_empty() {
            format!("position sfen {}\n", position_sfen)
        } else {
            format!("position sfen {} moves {}\n", 
                position_sfen.split(" moves").next().unwrap_or(position_sfen),
                moves.join(" ")
            )
        };

        stdin.write_all(pos_cmd.as_bytes()).await?;
        stdin.flush().await?;

        // Send go command
        let go_cmd = format!("go btime {} wtime {}\n", time_ms, time_ms);
        stdin.write_all(go_cmd.as_bytes()).await?;
        stdin.flush().await?;

        // Wait for bestmove
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let timeout_duration = Duration::from_secs(time_ms / 1000 + 10);
        let start = tokio::time::Instant::now();
        
        while start.elapsed() < timeout_duration {
            line.clear();
            
            match timeout(Duration::from_millis(100), reader.read_line(&mut line)).await {
                Ok(Ok(0)) => return Err(anyhow!("Engine closed connection")),
                Ok(Ok(_)) => {
                    let trimmed = line.trim();
                    log::debug!("Engine move response: {}", trimmed);
                    if trimmed.starts_with("bestmove ") {
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return Ok(parts[1].to_string());
                        }
                    }
                }
                Ok(Err(e)) => return Err(anyhow!("Failed to read from engine: {}", e)),
                Err(_) => continue, // Timeout, try again
            }
        }
        
        Err(anyhow!("Timeout waiting for bestmove"))
    }

    /// Run the engine-vs-engine match
    pub async fn run_match(mut self) -> Result<()> {
        log::info!("Starting engine-vs-engine match");

        // Spawn engines
        self.spawn_engines().await?;

        // Get stdin/stdout handles
        let engine1_stdin = self.engine1.as_mut()
            .and_then(|e| e.stdin.take())
            .ok_or_else(|| anyhow!("Failed to get engine 1 stdin"))?;
        let engine1_stdout = self.engine1.as_mut()
            .and_then(|e| e.stdout.take())
            .ok_or_else(|| anyhow!("Failed to get engine 1 stdout"))?;

        let engine2_stdin = self.engine2.as_mut()
            .and_then(|e| e.stdin.take())
            .ok_or_else(|| anyhow!("Failed to get engine 2 stdin"))?;
        let engine2_stdout = self.engine2.as_mut()
            .and_then(|e| e.stdout.take())
            .ok_or_else(|| anyhow!("Failed to get engine 2 stdout"))?;

        let mut engine1_stdin = engine1_stdin;
        let mut engine1_stdout = engine1_stdout;
        let mut engine2_stdin = engine2_stdin;
        let mut engine2_stdout = engine2_stdout;

        // Initialize both engines with saved options
        Self::initialize_engine_with_options(&mut engine1_stdin, &mut engine1_stdout, &self.config.engine1_id, &self.engine_storage).await?;
        Self::initialize_engine_with_options(&mut engine2_stdin, &mut engine2_stdout, &self.config.engine2_id, &self.engine_storage).await?;

        // Send usinewgame to both
        engine1_stdin.write_all(b"usinewgame\n").await?;
        engine1_stdin.flush().await?;
        engine2_stdin.write_all(b"usinewgame\n").await?;
        engine2_stdin.flush().await?;

        // Emit initial state
        {
            let state = self.state.lock().await;
            let _ = self.app_handle.emit("engine-vs-engine-update", state.clone());
        }

        // Main game loop
        for move_num in 1..=self.config.max_moves {
            let state_guard = self.state.lock().await;
            if state_guard.game_over {
                break;
            }
            let current_sfen = state_guard.position_sfen.clone();
            let move_history = state_guard.move_history.clone();
            let is_black_turn = state_guard.current_player == "black";
            drop(state_guard);

            // Select engine based on turn
            let (stdin, stdout, engine_name) = if is_black_turn {
                (&mut engine1_stdin, &mut engine1_stdout, &self.config.engine1_name)
            } else {
                (&mut engine2_stdin, &mut engine2_stdout, &self.config.engine2_name)
            };

            log::info!("Move {}: {} to move", move_num, if is_black_turn { "Black" } else { "White" });

            // Request move from engine
            let best_move = match Self::request_move(
                stdin,
                stdout,
                &current_sfen,
                &move_history,
                self.config.time_per_move_ms,
            ).await {
                Ok(mv) => mv,
                Err(e) => {
                    log::error!("Error getting move from {}: {}", engine_name, e);
                    // Engine error - opponent wins
                    let mut state = self.state.lock().await;
                    state.game_over = true;
                    state.winner = Some(if is_black_turn { "white".to_string() } else { "black".to_string() });
                    state.game_result = Some(format!("{} failed to respond", engine_name));
                    let _ = self.app_handle.emit("engine-vs-engine-update", state.clone());
                    break;
                }
            };

            // Check for resignation
            if best_move == "resign" {
                let mut state = self.state.lock().await;
                state.game_over = true;
                state.winner = Some(if is_black_turn { "white".to_string() } else { "black".to_string() });
                state.game_result = Some(format!("{} resigned", engine_name));
                let _ = self.app_handle.emit("engine-vs-engine-update", state.clone());
                log::info!("Game over: {} resigned", engine_name);
                break;
            }

            // Update state with new move
            {
                let mut state = self.state.lock().await;
                state.move_history.push(best_move.clone());
                state.last_move = Some(best_move.clone());
                state.current_player = if is_black_turn { "white".to_string() } else { "black".to_string() };
                state.move_number = move_num;
                
                // Update position SFEN to include all moves played
                let initial_sfen = current_sfen.split(" moves").next().unwrap_or(&current_sfen);
                if state.move_history.is_empty() {
                    state.position_sfen = initial_sfen.to_string();
                } else {
                    state.position_sfen = format!("{} moves {}", initial_sfen, state.move_history.join(" "));
                }

                // Emit update
                let _ = self.app_handle.emit("engine-vs-engine-update", state.clone());
                let _ = self.app_handle.emit("engine-vs-engine-move", serde_json::json!({
                    "move": best_move,
                    "engine": engine_name,
                    "move_number": move_num,
                }));
            }

            log::info!("{} played: {}", engine_name, best_move);

            // Small delay for UI updates
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        // Check if max moves reached
        {
            let mut state = self.state.lock().await;
            if !state.game_over {
                state.game_over = true;
                state.game_result = Some("Maximum moves reached".to_string());
                state.winner = Some("draw".to_string());
                let _ = self.app_handle.emit("engine-vs-engine-update", state.clone());
            }
        }

        // Cleanup engines
        let _ = engine1_stdin.write_all(b"quit\n").await;
        let _ = engine1_stdin.flush().await;
        let _ = engine2_stdin.write_all(b"quit\n").await;
        let _ = engine2_stdin.flush().await;

        if let Some(mut proc) = self.engine1.take() {
            let _ = proc.kill().await;
        }
        if let Some(mut proc) = self.engine2.take() {
            let _ = proc.kill().await;
        }

        log::info!("Engine-vs-engine match completed");
        Ok(())
    }
}

