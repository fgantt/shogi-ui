use std::io::{self, BufRead};
use std::thread;
use std::time::Duration;
use shogi_engine::UsiHandler;

fn main() {
    let mut handler = UsiHandler::new();

    for line in io::stdin().lock().lines() {
        let command = line.unwrap_or_else(|_| String::new());
        if command.trim() == "quit" {
            break;
        }

        let output = handler.handle_command(&command);
        for out_line in output {
            println!("{}", out_line);
        }

        if command.starts_with("go") {
            loop {
                thread::sleep(Duration::from_millis(100));
                let pending_output = handler.get_pending_output();
                let has_bestmove = pending_output.iter().any(|s| s.starts_with("bestmove"));
                for out_line in pending_output {
                    println!("{}", out_line);
                }
                if has_bestmove {
                    break;
                }
            }
        }
    }
}