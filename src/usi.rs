
use std::io::{self, BufRead, Write};
use crate::ShogiEngine;

pub struct UsiHandler {
    engine: ShogiEngine,
}

impl UsiHandler {
    pub fn new() -> Self {
        Self {
            engine: ShogiEngine::new(),
        }
    }

    pub fn handle_command(&mut self, command_str: &str) -> Vec<String> {
        let parts: Vec<&str> = command_str.trim().split_whitespace().collect();

        if parts.is_empty() {
            return Vec::new();
        }

        if self.engine.is_debug_mode() {
            // TODO: Add proper logging instead of returning debug messages.
        }

        match parts[0] {
            "usi" => self.handle_usi(),
            "isready" => self.handle_isready(),
            "debug" => self.engine.handle_debug(&parts[1..]),
            "position" => self.engine.handle_position(&parts[1..]),
            "go" => self.handle_go(&parts[1..]),
            "stop" => self.engine.handle_stop(),
            "ponderhit" => self.engine.handle_ponderhit(),
            "setoption" => self.engine.handle_setoption(&parts[1..]),
            "usinewgame" => self.engine.handle_usinewgame(),
            "gameover" => self.engine.handle_gameover(&parts[1..]),
            "quit" => Vec::new(), // quit is handled by the caller
            _ => vec![format!("info string Unknown command: {}", parts.join(" "))],
        }
    }

    fn handle_go(&mut self, parts: &[&str]) -> Vec<String> {
        crate::debug_utils::trace_log("USI_GO", "Starting go command processing");
        crate::debug_utils::set_search_start_time();
        crate::debug_utils::start_timing("go_command_parsing");
        
        let mut btime = 0;
        let mut wtime = 0;
        let mut byoyomi = 0;

        let mut i = 0;
        while i < parts.len() {
            match parts[i] {
                "btime" => {
                    if i + 1 < parts.len() {
                        btime = parts[i+1].parse().unwrap_or(0);
                        i += 2;
                    } else { i += 1; }
                },
                "wtime" => {
                    if i + 1 < parts.len() {
                        wtime = parts[i+1].parse().unwrap_or(0);
                        i += 2;
                    }
                    else { i += 1; }
                },
                "byoyomi" => {
                    if i + 1 < parts.len() {
                        byoyomi = parts[i+1].parse().unwrap_or(0);
                        i += 2;
                    } else { i += 1; }
                },
                _ => i += 1,
            }
        }

        crate::debug_utils::end_timing("go_command_parsing", "USI_GO");
        crate::debug_utils::trace_log("USI_GO", &format!("Parsed time controls: btime={}ms wtime={}ms byoyomi={}ms", btime, wtime, byoyomi));

        let time_to_use = if byoyomi > 0 {
            byoyomi
        } else {
            let time_for_player = if self.engine.current_player == crate::types::Player::Black { btime } else { wtime };
            if time_for_player > 0 {
                time_for_player / 40 // Use a fraction of the remaining time
            } else {
                5000 // Default to 5 seconds if no time control is given
            }
        };

        crate::debug_utils::log_decision("USI_GO", "Time allocation", 
            &format!("Player: {:?}, Allocated time: {}ms", self.engine.current_player, time_to_use), 
            Some(time_to_use as i32));

        self.engine.stop_flag.store(false, std::sync::atomic::Ordering::Relaxed);

        crate::debug_utils::start_timing("best_move_search");
        let best_move = self.engine.get_best_move(
            self.engine.depth,
            time_to_use,
            Some(self.engine.stop_flag.clone()),
        );
        crate::debug_utils::end_timing("best_move_search", "USI_GO");

        if let Some(mv) = best_move {
            crate::debug_utils::trace_log("USI_GO", &format!("Best move found: {}", mv.to_usi_string()));
            vec![format!("bestmove {}", mv.to_usi_string())]
        } else {
            crate::debug_utils::trace_log("USI_GO", "No legal moves found, resigning");
            vec!["bestmove resign".to_string()]
        }
    }

    fn handle_usi(&self) -> Vec<String> {
        vec![
            "id name Shogi Engine".to_string(),
            "id author Gemini".to_string(),
            "option name USI_Hash type spin default 16 min 1 max 1024".to_string(),
            "option name depth type spin default 5 min 1 max 8".to_string(),
            "usiok".to_string(),
        ]
    }

    fn handle_isready(&self) -> Vec<String> {
        vec!["readyok".to_string()]
    }
}

pub fn run_usi_loop() {
    let mut handler = UsiHandler::new();
    let mut stdout = io::stdout();

    for line in io::stdin().lock().lines() {
        let command = line.unwrap_or_else(|_| String::new());
        if command.trim() == "quit" {
            break;
        }

        let output = handler.handle_command(&command);
        for out_line in output {
            if let Err(e) = writeln!(stdout, "{}", out_line) {
                eprintln!("Error writing to stdout: {}", e);
                return;
            }
        }
        if let Err(e) = stdout.flush() {
            eprintln!("Error flushing stdout: {}", e);
            return;
        }
    }
}
