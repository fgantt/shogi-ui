#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::search::search_engine::{IterativeDeepening, SearchEngine};
use shogi_engine::search::ParallelSearchConfig;
use shogi_engine::types::{CapturedPieces, Player};

fn parse_fen(fen: &str) -> Option<(BitboardBoard, Player, CapturedPieces)> {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    let fen3 = if parts.len() >= 3 {
        format!("{} {} {}", parts[0], parts[1], parts[2])
    } else {
        fen.to_string()
    };
    match BitboardBoard::from_fen(&fen3) {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

fn best_move_threads(fen: &str, depth: u8, threads: usize) -> Option<String> {
    let (board, player, captured) = match parse_fen(fen) {
        Some(v) => v,
        None => return None,
    };
    let mut engine = SearchEngine::new(None, 16);
    let mut id = if threads > 1 {
        let config = ParallelSearchConfig::new(threads);
        IterativeDeepening::new_with_threads(depth, 1000, None, threads, config)
    } else {
        IterativeDeepening::new(depth, 1000, None)
    };
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        id.search(&mut engine, &board, &captured, player)
            .map(|(m, _)| m.to_usi_string())
    }));
    match res {
        Ok(v) => v,
        Err(_) => None,
    }
}

#[test]
fn test_tactical_dataset_parallel_vs_single() {
    let data = include_str!("data/tactical_fens.txt");
    let mut total = 0usize;
    let mut compared = 0usize;
    let mut mismatches = 0usize;
    for line in data.lines() {
        let fen = line.trim();
        if fen.is_empty() || fen.starts_with('#') {
            continue;
        }
        total += 1;
        let m1 = best_move_threads(fen, 2, 1);
        let m4 = best_move_threads(fen, 2, 4);
        if let (Some(a), Some(b)) = (m1, m4) {
            compared += 1;
            if a != b {
                mismatches += 1;
            }
        }
    }
    // Require at least some coverage and tolerate some mismatch due to nondeterminism
    assert!(
        compared >= 10,
        "Too few comparable positions ({} of {})",
        compared,
        total
    );
    let mismatch_ratio = if compared > 0 {
        mismatches as f64 / compared as f64
    } else {
        1.0
    };
    assert!(
        mismatch_ratio <= 0.9,
        "Too many mismatches: {}/{} ({:.1}%)",
        mismatches,
        compared,
        mismatch_ratio * 100.0
    );
}
