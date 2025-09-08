use std::io::{self, BufRead};
use shogi_engine::ShogiEngine;

fn main() {
    let mut engine = ShogiEngine::new();

    for line in io::stdin().lock().lines() {
        let command = line.unwrap_or_else(|_| String::new());
        let parts: Vec<&str> = command.trim().split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "usi" => handle_usi(),
            "isready" => handle_isready(),
            "position" => engine.handle_position(&parts[1..]),
            "go" => engine.handle_go(&parts[1..]),
            "stop" => engine.handle_stop(),
            "setoption" => engine.handle_setoption(&parts[1..]),
            "usinewgame" => engine.handle_usinewgame(),
            "quit" => break,
            _ => {
                println!("info string Unknown command: {}", command);
            }
        }
    }
}

fn handle_usi() {
    println!("id name Shogi Engine");
    println!("id author Gemini");
    println!("option name USI_Hash type spin default 16 min 1 max 1024");
    println!("usiok");
}

fn handle_isready() {
    println!("readyok");
}