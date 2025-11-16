use crate::ShogiEngine;
use num_cpus;
use std::io::{self, BufRead, Write};

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
        crate::utils::telemetry::trace_log("USI_GO", "Starting go command processing");
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
                        btime = parts[i + 1].parse().unwrap_or(0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "wtime" => {
                    if i + 1 < parts.len() {
                        wtime = parts[i + 1].parse().unwrap_or(0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "byoyomi" => {
                    if i + 1 < parts.len() {
                        byoyomi = parts[i + 1].parse().unwrap_or(0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }

        crate::debug_utils::end_timing("go_command_parsing", "USI_GO");
        crate::utils::telemetry::trace_log(
            "USI_GO",
            &format!(
                "Parsed time controls: btime={}ms wtime={}ms byoyomi={}ms",
                btime, wtime, byoyomi
            ),
        );

        let time_to_use = if byoyomi > 0 {
            byoyomi
        } else {
            let time_for_player = if self.engine.current_player == crate::types::Player::Black {
                btime
            } else {
                wtime
            };
            if time_for_player > 0 {
                time_for_player / 40 // Use a fraction of the remaining time
            } else {
                5000 // Default to 5 seconds if no time control is given
            }
        };

        crate::debug_utils::log_decision(
            "USI_GO",
            "Time allocation",
            &format!(
                "Player: {:?}, Allocated time: {}ms",
                self.engine.current_player, time_to_use
            ),
            Some(time_to_use as i32),
        );

        self.engine
            .stop_flag
            .store(false, std::sync::atomic::Ordering::Relaxed);

        crate::debug_utils::start_timing("best_move_search");
        let best_move = self.engine.get_best_move(
            self.engine.depth,
            time_to_use,
            Some(self.engine.stop_flag.clone()),
        );
        crate::debug_utils::end_timing("best_move_search", "USI_GO");

        if let Some(mv) = best_move {
            crate::utils::telemetry::trace_log(
                "USI_GO",
                &format!("Best move found: {}", mv.to_usi_string()),
            );
            vec![format!("bestmove {}", mv.to_usi_string())]
        } else {
            crate::utils::telemetry::trace_log("USI_GO", "No legal moves found, resigning");
            vec!["bestmove resign".to_string()]
        }
    }

    fn handle_usi(&self) -> Vec<String> {
        let thread_count = num_cpus::get();
        let parallel_options = self.engine.parallel_search_options();
        vec![
            "id name Shogi Engine".to_string(),
            "id author Gemini".to_string(),
            "option name USI_Hash type spin default 16 min 1 max 1024".to_string(),
            format!(
                "option name ParallelEnable type check default {}",
                if parallel_options.enable_parallel {
                    "true"
                } else {
                    "false"
                }
            ),
            format!(
                "option name ParallelHash type spin default {} min 1 max 512",
                parallel_options.hash_size_mb
            ),
            format!(
                "option name ParallelMinDepth type spin default {} min 0 max 32",
                parallel_options.min_depth_parallel
            ),
            "option name PSTPreset type combo default Builtin var Builtin var Default var Custom"
                .to_string(),
            "option name PSTPath type string default".to_string(),
            format!(
                "option name ParallelMetrics type check default {}",
                if parallel_options.enable_metrics {
                    "true"
                } else {
                    "false"
                }
            ),
            format!(
                "option name YBWCEnable type check default {}",
                if parallel_options.ybwc_enabled {
                    "true"
                } else {
                    "false"
                }
            ),
            format!(
                "option name YBWCMinDepth type spin default {} min 0 max 32",
                parallel_options.ybwc_min_depth
            ),
            format!(
                "option name YBWCMinBranch type spin default {} min 1 max 256",
                parallel_options.ybwc_min_branch
            ),
            format!(
                "option name YBWCMaxSiblings type spin default {} min 1 max 256",
                parallel_options.ybwc_max_siblings
            ),
            format!(
                "option name YBWCScalingShallow type spin default {} min 1 max 32",
                parallel_options.ybwc_shallow_divisor
            ),
            format!(
                "option name YBWCScalingMid type spin default {} min 1 max 32",
                parallel_options.ybwc_mid_divisor
            ),
            format!(
                "option name YBWCScalingDeep type spin default {} min 1 max 32",
                parallel_options.ybwc_deep_divisor
            ),
            // Fixed: MaxDepth now allows 0-100 (0 = unlimited/adaptive), default 0
            "option name MaxDepth type spin default 0 min 0 max 100".to_string(),
            format!("option name USI_Threads type spin default {} min 1 max 32", thread_count),
            // Time Management Options (Task 8.0, 4.0)
            "option name TimeCheckFrequency type spin default 1024 min 1 max 100000".to_string(),
            "option name TimeSafetyMargin type spin default 100 min 0 max 10000".to_string(),
            "option name TimeAllocationStrategy type combo default Adaptive var Equal var Exponential var Adaptive".to_string(),
            "option name EnableTimeBudget type check default true".to_string(),
            "option name EnableCheckOptimization type check default true".to_string(),
            // Aspiration Window Options (Task 7.0)
            "option name EnableAspirationWindows type check default true".to_string(),
            "option name AspirationWindowSize type spin default 25 min 10 max 500".to_string(),
            "option name EnablePositionTypeTracking type check default true".to_string(),
            // Legacy depth option (for backward compatibility, maps to MaxDepth)
            "option name depth type spin default 0 min 0 max 100".to_string(),
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
