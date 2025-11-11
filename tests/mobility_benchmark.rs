use std::time::Instant;

use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::position_features::PositionFeatureEvaluator;
use shogi_engine::moves::MoveGenerator;
use shogi_engine::types::{CapturedPieces, PieceType, Player, Position, TaperedScore};

#[ignore]
#[test]
fn mobility_benchmark_snapshot() {
    let iterations = 5_000;

    let mut evaluator = PositionFeatureEvaluator::new();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();

    // Warm up caches
    for _ in 0..512 {
        let _ = evaluator.evaluate_mobility(&board, Player::Black, &captured);
        let _ = evaluate_mobility_naive(&board, Player::Black, &captured);
    }

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = evaluator.evaluate_mobility(&board, Player::Black, &captured);
    }
    let cached_duration = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = evaluate_mobility_naive(&board, Player::Black, &captured);
    }
    let naive_duration = start.elapsed();

    println!(
        "iterations={}, cached={:.2?}, naive={:.2?}, speedup={:.2}x",
        iterations,
        cached_duration,
        naive_duration,
        naive_duration.as_secs_f64() / cached_duration.as_secs_f64()
    );
}

fn evaluate_mobility_naive(
    board: &BitboardBoard,
    player: Player,
    captured: &CapturedPieces,
) -> TaperedScore {
    let mut mg_total = 0;
    let mut eg_total = 0;
    for row in 0..9 {
        for col in 0..9 {
            let pos = Position::new(row, col);
            if let Some(piece) = board.get_piece(pos) {
                if piece.player != player {
                    continue;
                }
                let generator = MoveGenerator::new();
                let moves = generator.generate_legal_moves(board, player, captured);
                let mut move_count = 0;
                let mut central_moves = 0;
                let mut capture_moves = 0;
                for mv in moves.iter().filter(|m| m.from == Some(pos)) {
                    move_count += 1;
                    if is_central_square(mv.to) {
                        central_moves += 1;
                    }
                    if mv.is_capture {
                        capture_moves += 1;
                    }
                }
                let (mg, eg) = evaluate_piece_mobility(
                    piece.piece_type,
                    move_count,
                    central_moves,
                    capture_moves,
                );
                mg_total += mg;
                eg_total += eg;
            }
        }
    }
    TaperedScore::new_tapered(mg_total, eg_total)
}

fn evaluate_piece_mobility(
    piece_type: PieceType,
    move_count: i32,
    central_moves: i32,
    capture_moves: i32,
) -> (i32, i32) {
    let (mg_weight, eg_weight) = mobility_weight(piece_type);
    let mut mg_score = move_count * mg_weight;
    let mut eg_score = move_count * eg_weight;

    if move_count <= 2 {
        let (mg_penalty, eg_penalty) = restriction_penalty(piece_type);
        mg_score -= mg_penalty;
        eg_score -= eg_penalty;
    }

    if central_moves > 0 {
        let (mg_bonus, eg_bonus) = central_bonus(piece_type);
        mg_score += central_moves * mg_bonus;
        eg_score += central_moves * eg_bonus;
    }

    if capture_moves > 0 {
        mg_score += capture_moves * 4;
        eg_score += capture_moves * 3;
    }

    (mg_score, eg_score)
}

fn mobility_weight(piece_type: PieceType) -> (i32, i32) {
    match piece_type {
        PieceType::Rook => (5, 7),
        PieceType::PromotedRook => (6, 8),
        PieceType::Bishop => (4, 6),
        PieceType::PromotedBishop => (5, 7),
        PieceType::Gold => (2, 3),
        PieceType::Silver => (2, 3),
        PieceType::Knight => (3, 3),
        PieceType::Lance => (2, 2),
        PieceType::PromotedPawn
        | PieceType::PromotedLance
        | PieceType::PromotedKnight
        | PieceType::PromotedSilver => (3, 4),
        PieceType::Pawn => (1, 1),
        PieceType::King => (1, 2),
    }
}

fn restriction_penalty(piece_type: PieceType) -> (i32, i32) {
    match piece_type {
        PieceType::Rook | PieceType::PromotedRook => (18, 24),
        PieceType::Bishop | PieceType::PromotedBishop => (16, 22),
        PieceType::Gold | PieceType::Silver => (6, 8),
        PieceType::Knight | PieceType::Lance => (7, 9),
        PieceType::PromotedPawn
        | PieceType::PromotedLance
        | PieceType::PromotedKnight
        | PieceType::PromotedSilver => (6, 8),
        PieceType::Pawn => (3, 4),
        PieceType::King => (4, 6),
    }
}

fn central_bonus(piece_type: PieceType) -> (i32, i32) {
    match piece_type {
        PieceType::Rook | PieceType::PromotedRook => (4, 3),
        PieceType::Bishop | PieceType::PromotedBishop => (3, 3),
        PieceType::Knight => (4, 2),
        PieceType::Gold | PieceType::Silver => (2, 2),
        PieceType::Lance => (1, 1),
        PieceType::PromotedPawn
        | PieceType::PromotedLance
        | PieceType::PromotedKnight
        | PieceType::PromotedSilver => (3, 2),
        PieceType::Pawn => (1, 1),
        PieceType::King => (1, 1),
    }
}

fn is_central_square(pos: Position) -> bool {
    pos.row >= 3 && pos.row <= 5 && pos.col >= 3 && pos.col <= 5
}
