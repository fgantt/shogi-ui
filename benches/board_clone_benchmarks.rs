use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use shogi_engine::bitboards::{BitboardBoard, get_board_telemetry, reset_board_telemetry};
use shogi_engine::moves::MoveGenerator;
use shogi_engine::types::{CapturedPieces, Player, Position};

/// Task 5.0.5.1: Extended board cloning benchmarks
fn bench_board_cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("board_clone");
    group.sampling_mode(SamplingMode::Auto);

    reset_board_telemetry();
    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let mg = MoveGenerator::new();
    let legal = mg.generate_legal_moves(&board, player, &captured);

    group.bench_with_input(
        BenchmarkId::new("BitboardBoard::clone", legal.len()),
        &legal.len(),
        |b, _| {
            b.iter(|| {
                let _b2 = black_box(board.clone());
            });
        },
    );

    group.bench_function("CapturedPieces::clone", |b| {
        b.iter(|| {
            let _c2 = black_box(captured.clone());
        });
    });

    // Clone + make_move typical root pattern
    if let Some(first) = legal.get(0) {
        group.bench_function("clone_then_make_move", |b| {
            b.iter(|| {
                let mut b2 = board.clone();
                let mut c2 = captured.clone();
                if let Some(capt) = b2.make_move(first) {
                    c2.add_piece(capt.piece_type, player);
                }
                black_box((b2, c2));
            });
        });
    }

    // Task 5.0.5.1: Benchmark multiple sequential clones (common in search)
    group.bench_function("sequential_clones_10", |b| {
        b.iter(|| {
            let mut boards = Vec::new();
            for _ in 0..10 {
                boards.push(black_box(board.clone()));
            }
            black_box(boards);
        });
    });

    group.finish();
    
    // Task 5.0.5.1: Report telemetry after cloning benchmarks
    let telemetry = get_board_telemetry();
    println!("Board clone telemetry: {} clones performed", telemetry.clone_count);
}

/// Task 5.0.5.1: Benchmark legal move generation
fn bench_legal_move_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("legal_move_generation");
    group.sampling_mode(SamplingMode::Auto);

    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let mg = MoveGenerator::new();

    for player in [Player::Black, Player::White] {
        group.bench_function(format!("generate_legal_moves_{:?}", player), |b| {
            b.iter(|| {
                let moves = mg.generate_legal_moves(&board, player, &captured);
                black_box(moves);
            });
        });
    }

    // Benchmark move generation from different positions
    let mut test_board = BitboardBoard::empty();
    test_board.setup_initial_position();
    group.bench_function("generate_from_startpos", |b| {
        b.iter(|| {
            let moves = mg.generate_legal_moves(&test_board, Player::Black, &captured);
            black_box(moves);
        });
    });

    group.finish();
}

/// Task 5.0.5.1: Benchmark attack detection
fn bench_attack_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("attack_detection");
    group.sampling_mode(SamplingMode::Auto);

    let board = BitboardBoard::new();
    
    // Test different squares
    let test_squares = vec![
        Position::new(4, 4), // Center
        Position::new(0, 0), // Corner
        Position::new(0, 4), // Edge
    ];

    for square in test_squares {
        for player in [Player::Black, Player::White] {
            group.bench_function(
                format!("is_square_attacked_by_{:?}_{:?}", square, player),
                |b| {
                    b.iter(|| {
                        let result = board.is_square_attacked_by(square, player);
                        black_box(result);
                    });
                },
            );
        }
    }

    // Benchmark attack detection for all squares
    group.bench_function("is_square_attacked_all_squares", |b| {
        b.iter(|| {
            let mut count = 0;
            for row in 0..9 {
                for col in 0..9 {
                    let pos = Position::new(row, col);
                    if board.is_square_attacked_by(pos, Player::Black) {
                        count += 1;
                    }
                }
            }
            black_box(count);
        });
    });

    group.finish();
}

/// Task 5.0.5.1: Benchmark sliding move generation
fn bench_sliding_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("sliding_moves");
    group.sampling_mode(SamplingMode::Auto);

    let mut board = BitboardBoard::new();
    
    // Initialize sliding generator if available
    if let Ok(_) = board.init_sliding_generator() {
        use shogi_engine::types::{Piece, PieceType};
        
        // Place a rook and bishop for testing
        let rook_pos = Position::new(4, 4);
        let bishop_pos = Position::new(4, 3);
        board.place_piece(Piece::new(PieceType::Rook, Player::Black), rook_pos);
        board.place_piece(Piece::new(PieceType::Bishop, Player::Black), bishop_pos);

        if let Some(generator) = board.get_sliding_generator() {
            group.bench_function("generate_sliding_moves_rook", |b| {
                b.iter(|| {
                    let moves = generator.generate_sliding_moves(&board, rook_pos, PieceType::Rook, Player::Black);
                    black_box(moves);
                });
            });

            group.bench_function("generate_sliding_moves_bishop", |b| {
                b.iter(|| {
                    let moves = generator.generate_sliding_moves(&board, bishop_pos, PieceType::Bishop, Player::Black);
                    black_box(moves);
                });
            });
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_board_cloning,
    bench_legal_move_generation,
    bench_attack_detection,
    bench_sliding_moves
);
criterion_main!(benches);
