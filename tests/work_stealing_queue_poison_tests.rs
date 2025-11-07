#![cfg(feature = "legacy-tests")]
use shogi_engine::search::WorkStealingQueue;
use shogi_engine::types::{Move, PieceType, Player, Position};

fn sample_work() -> shogi_engine::search::WorkUnit {
    use shogi_engine::bitboards::BitboardBoard;
    use shogi_engine::types::CapturedPieces;
    shogi_engine::search::WorkUnit {
        board: BitboardBoard::new(),
        captured_pieces: CapturedPieces::new(),
        move_to_search: Move {
            from: None,
            to: Position::from_u8(0),
            piece_type: PieceType::Pawn,
            player: Player::Black,
            is_promotion: false,
            is_capture: false,
            captured_piece: None,
            gives_check: false,
            is_recapture: false,
        },
        depth: 1,
        alpha: -1000,
        beta: 1000,
        parent_score: 0,
        player: Player::Black,
        time_limit_ms: 10,
        is_oldest_brother: true,
    }
}

#[test]
fn test_queue_recovers_from_poison_in_push_and_pop() {
    let q = WorkStealingQueue::new();
    // Poison the lock by panicking while holding it
    q.test_poison();
    // Now push and pop should still succeed due to recovery
    q.push_back(sample_work());
    let _ = q.pop_front();
}
