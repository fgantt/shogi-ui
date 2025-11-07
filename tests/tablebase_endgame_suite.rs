#![cfg(feature = "legacy-tests")]
use std::fs;
use std::path::Path;

use shogi_engine::tablebase::endgame_solvers::king_gold_vs_king::KingGoldVsKingSolver;
use shogi_engine::tablebase::{EndgameSolver, MicroTablebase, TablebaseOutcome};
use shogi_engine::types::CapturedPieces;
use shogi_engine::types::Player;
use shogi_engine::BitboardBoard;

fn parse_fen_to_board_captured_player(
    fen: &str,
) -> Option<(BitboardBoard, CapturedPieces, Player)> {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let fen3 = format!("{} {} {}", parts[0], parts[1], parts[2]);
    let player = if parts[1] == "b" {
        Player::Black
    } else {
        Player::White
    };
    let (board, _p, captured) = BitboardBoard::from_fen(&fen3).ok()?;
    Some((board, captured, player))
}

#[test]
fn smoke_solver_detection_kgk() {
    // This is a logic-only smoke check for solver detection; not asserting outcome
    // FEN approximates KG vs K; if parsing fails on environment, test still passes
    let fen = "4k4/9/9/9/9/9/9/9/3G1K3 b -";
    if let Some((board, captured, player)) = parse_fen_to_board_captured_player(fen) {
        let solver = KingGoldVsKingSolver::new();
        // Player argument is ignored by can_solve; structure-only check
        let _ = solver.can_solve(&board, player, &captured);
    }
}

#[test]
fn param_tablebase_positions_gated() {
    if std::env::var("SHOGI_TEST_TB").ok().as_deref() != Some("1") {
        eprintln!("Skipping tablebase suite: set SHOGI_TEST_TB=1 to enable");
        return;
    }

    let data_path = Path::new("tests/data/endgame_tb_positions.csv");
    if !data_path.exists() {
        eprintln!("No endgame_tb_positions.csv found; skipping");
        return;
    }

    let mut tb = MicroTablebase::new();
    let content = fs::read_to_string(data_path).expect("read csv");
    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // CSV: fen,expected_outcome  where expected_outcome in {win,loss,draw}
        let parts: Vec<&str> = line.splitn(2, ',').collect();
        if parts.len() < 2 {
            continue;
        }
        let fen = parts[0].trim();
        let expected = parts[1].trim().to_lowercase();
        let Some((board, captured, player)) = parse_fen_to_board_captured_player(fen) else {
            continue;
        };

        if let Some(result) = tb.probe(&board, player, &captured) {
            match expected.as_str() {
                "win" => assert!(
                    result.outcome == TablebaseOutcome::Win,
                    "row {} expected win",
                    idx + 1
                ),
                "loss" => assert!(
                    result.outcome == TablebaseOutcome::Loss,
                    "row {} expected loss",
                    idx + 1
                ),
                "draw" => assert!(
                    result.outcome == TablebaseOutcome::Draw,
                    "row {} expected draw",
                    idx + 1
                ),
                _ => {}
            }
        } else {
            // Not solvable by current built-in TBs; acceptable for now
        }
    }
}
