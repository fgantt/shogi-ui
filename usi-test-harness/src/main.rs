use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Duration;
use std::{thread, env};

use anyhow::{anyhow, Result};
use regex::Regex;

mod fen_util;
use fen_util::{BoardState, Player};

struct UsiEngine {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    bestmove_regex: Regex,
}

impl UsiEngine {
    fn new(engine_path: &str) -> Result<Self> {
        let mut child = Command::new(engine_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("Failed to open stdin"))?;
        let stdout = BufReader::new(child.stdout.take().ok_or_else(|| anyhow!("Failed to open stdout"))?);

        let bestmove_regex = Regex::new(r"bestmove\s+(\S+)")?;

        let mut engine = UsiEngine {
            child,
            stdin,
            stdout,
            bestmove_regex,
        };

        engine.send_command("usi")?;
        engine.read_response("usiok")?;
        engine.send_command("isready")?;
        engine.read_response("readyok")?;

        Ok(engine)
    }

    fn send_command(&mut self, command: &str) -> Result<()> {
        writeln!(self.stdin, "{}", command)?;
        Ok(())
    }

    fn read_response(&mut self, expected_response: &str) -> Result<String> {
        let mut line = String::new();
        loop {
            line.clear();
            self.stdout.read_line(&mut line)?;
            let trimmed_line = line.trim();
            if trimmed_line.contains(expected_response) {
                return Ok(trimmed_line.to_string());
            }
            // Optionally, log other lines for debugging
            println!("Engine: {}", trimmed_line);
            std::io::stdout().flush()?;
        }
    }

    fn get_bestmove(&mut self, player_prefix: &str) -> Result<String> {
        self.send_command("go infinite")?; // Or go depth X, go movetime Y

        let mut line = String::new();
        loop {
            line.clear();
            self.stdout.read_line(&mut line)?;
            let trimmed_line = line.trim();
            println!("{}", trimmed_line); // Print engine response
            std::io::stdout().flush()?;

            if let Some(captures) = self.bestmove_regex.captures(trimmed_line) {
                let move_str = captures.get(1).unwrap().as_str().to_string();
                return Ok(move_str);
            }
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let engine_path = args.get(1).ok_or_else(|| anyhow!("Usage: usi-test-harness <path_to_shogi_engine>"))?;

    let mut engine = UsiEngine::new(engine_path)?;

    let mut current_fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1".to_string(); // Startpos FEN
    let mut board_state = BoardState::parse_fen(&current_fen)?;

    loop {
        let player_prefix = if board_state.current_player == Player::Black { "b" } else { "w" };
        let position_command = format!("position sfen {}", current_fen);
        println!("{}> {}", player_prefix, position_command);
        engine.send_command(&position_command)?;

        let best_move_usi_str = engine.get_bestmove(player_prefix)?;
        println!("Best move from engine: {}", best_move_usi_str);

        if best_move_usi_str == "resign" {
            println!("Game over: {} resigned", player_prefix);
            break;
        }

        board_state.apply_move(&best_move_usi_str)?;
        println!("Captured pieces: Black: {:?}, White: {:?}", board_state.black_captured, board_state.white_captured);
        current_fen = board_state.to_fen();

        let mut move_count = 0;
        // Placeholder for game end condition
        if board_state.current_player == Player::Black && current_fen == "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1" { // Simple check for repetition
            println!("Game over: Repetition");
            break;
        }
        move_count += 1;
        if move_count > 10 { // Play 10 moves for testing
            println!("Game over: Reached 10 moves");
            break;
        }

        thread::sleep(Duration::from_millis(100)); // Simulate thinking time
    }

    Ok(())
}