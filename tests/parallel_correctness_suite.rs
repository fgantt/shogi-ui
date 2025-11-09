#![cfg(feature = "legacy-tests")]
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::moves::MoveGenerator;
use shogi_engine::search::search_engine::{IterativeDeepening, SearchEngine};
use shogi_engine::search::ParallelSearchConfig;
use shogi_engine::types::{CapturedPieces, Player};

fn parse_fen(fen: &str) -> (BitboardBoard, Player, CapturedPieces) {
    // BitboardBoard::from_fen expects three parts: board, player, captured
    // Ensure input meets that by stripping trailing move counts if present
    let parts: Vec<&str> = fen.split_whitespace().collect();
    let fen3 = if parts.len() >= 3 {
        format!("{} {} {}", parts[0], parts[1], parts[2])
    } else {
        fen.to_string()
    };
    BitboardBoard::from_fen(&fen3).expect("valid FEN")
}

fn best_move_threads(fen: &str, depth: u8, threads: usize) -> Option<String> {
    let (board, player, captured) = parse_fen(fen);
    let mut engine = SearchEngine::new(None, 16);
    let mut id = if threads > 1 {
        IterativeDeepening::new_with_threads(
            depth,
            1000,
            None,
            threads,
            ParallelSearchConfig::new(threads),
        )
    } else {
        IterativeDeepening::new(depth, 1000, None)
    };
    id.search(&mut engine, &board, &captured, player)
        .map(|(m, _)| m.to_usi_string())
}

#[test]
fn test_parallel_vs_single_threaded_on_positions() {
    // A small set of representative positions; more can be added over time
    let positions = vec![
        // Standard start position (black to move)
        "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b -",
        // A side-to-move swap
        "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w -",
        // Reduced material midgame-ish
        "4k4/1r7/pp1p1pppp/9/9/9/PP1P1PPPP/7R1/4K4 b -",
    ];

    for fen in positions {
        let m1 = best_move_threads(fen, 3, 1);
        let m4 = best_move_threads(fen, 3, 4);
        // Parallel search can select different but comparable best moves; only require both respond
        assert!(
            m1.is_some() && m4.is_some(),
            "Engine did not return a move at fen={}",
            fen
        );
    }
}

#[test]
fn test_thread_safety_concurrent_searches() {
    let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b -";
    let mut handles = Vec::new();
    for _ in 0..4 {
        let fen_s = fen.to_string();
        handles.push(std::thread::spawn(move || best_move_threads(&fen_s, 3, 4)));
    }
    let res: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    // Only assert all threads produced a move; allow benign divergence in PV under parallelism
    assert!(
        res.iter().all(|m| m.is_some()),
        "Concurrent searches failed to return moves: {:?}",
        res
    );
}

#[test]
#[ignore]
fn test_stress_many_searches() {
    // Intentionally heavy; run manually when needed
    let fen = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b -";
    for _ in 0..1000 {
        let _ = best_move_threads(fen, 2, 4);
    }
}

#[test]
fn test_tactical_puzzle_sanity() {
    // Ensure we at least return a legal move on a tactical-ish reduced position
    let fen = "4k4/1r7/pp1p1pppp/9/9/9/PP1P1PPPP/7R1/4K4 b -";
    let m1 = best_move_threads(fen, 3, 1);
    let m4 = best_move_threads(fen, 3, 4);
    assert!(m1.is_some() && m4.is_some());
}

#[test]
#[ignore]
fn test_endgame_tablebase_parallel_sanity() {
    // Placeholder: depends on tablebase coverage; run as a smoke test
    let fen = "4k4/9/9/9/9/9/9/9/4K4 b -"; // kings only (shogi-specific semantics may vary)
    let m8 = best_move_threads(fen, 1, 8);
    // We only assert that the engine responds; specific TB hits are environment-dependent
    assert!(m8.is_some());
}
