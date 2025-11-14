use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::castle_fixtures::castle_fixtures;
use shogi_engine::evaluation::castles::CastleRecognizer;
use shogi_engine::evaluation::king_safety::KingSafetyEvaluator;
use shogi_engine::types::{Piece, PieceType, Player, Position};

/// Create a test board with a Mino castle
fn create_mino_castle_board(
    player: Player,
    king_row: u8,
    king_col: u8,
) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_pos = Position::new(king_row, king_col);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);

    // Place gold and silver pieces for Mino castle
    if let Some(pos) = relative_pos(king_pos, player, -1, 0) {
        board.place_piece(Piece::new(PieceType::Gold, player), pos);
    }
    if let Some(pos) = relative_pos(king_pos, player, -2, 0) {
        board.place_piece(Piece::new(PieceType::Silver, player), pos);
    }

    // Place pawns for wall
    if let Some(pos) = relative_pos(king_pos, player, -2, -1) {
        board.place_piece(Piece::new(PieceType::Pawn, player), pos);
    }
    if let Some(pos) = relative_pos(king_pos, player, -2, 1) {
        board.place_piece(Piece::new(PieceType::Pawn, player), pos);
    }

    (board, king_pos)
}

/// Create a test board with an Anaguma castle
fn create_anaguma_castle_board(
    player: Player,
    king_row: u8,
    king_col: u8,
) -> (BitboardBoard, Position) {
    let mut board = BitboardBoard::empty();
    let king_pos = Position::new(king_row, king_col);

    board.place_piece(Piece::new(PieceType::King, player), king_pos);

    // Place gold and silver pieces for Anaguma castle
    if let Some(pos) = relative_pos(king_pos, player, -1, -1) {
        board.place_piece(Piece::new(PieceType::Gold, player), pos);
    }
    if let Some(pos) = relative_pos(king_pos, player, -1, 0) {
        board.place_piece(Piece::new(PieceType::Gold, player), pos);
    }
    if let Some(pos) = relative_pos(king_pos, player, -2, -1) {
        board.place_piece(Piece::new(PieceType::Silver, player), pos);
    }

    // Place pawns for wall
    if let Some(pos) = relative_pos(king_pos, player, -2, -2) {
        board.place_piece(Piece::new(PieceType::Pawn, player), pos);
    }
    if let Some(pos) = relative_pos(king_pos, player, -2, 0) {
        board.place_piece(Piece::new(PieceType::Pawn, player), pos);
    }

    (board, king_pos)
}

/// Helper to calculate relative position
fn relative_pos(
    king: Position,
    player: Player,
    rank_delta: i8,
    file_delta: i8,
) -> Option<Position> {
    let (rank_delta, file_delta) = match player {
        Player::Black => (rank_delta, file_delta),
        Player::White => (-rank_delta, -file_delta),
    };

    let new_rank = king.row as i8 + rank_delta;
    let new_file = king.col as i8 + file_delta;

    if (0..9).contains(&new_rank) && (0..9).contains(&new_file) {
        Some(Position::new(new_rank as u8, new_file as u8))
    } else {
        None
    }
}

fn castle_recognition_cache_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("castle_recognition_cache");

    // Benchmark cache hit rate with repeated evaluations
    group.bench_function("cache_hit_rate_repeated_eval", |b| {
        let recognizer = CastleRecognizer::new();
        let (board, king_pos) = create_mino_castle_board(Player::Black, 8, 4);

        // Warm up cache
        recognizer.evaluate_castle(&board, Player::Black, king_pos);

        b.iter(|| {
            recognizer.evaluate_castle(black_box(&board), Player::Black, black_box(king_pos));
        });
    });

    // Benchmark cache miss rate with varied positions
    group.bench_function("cache_miss_rate_varied_positions", |b| {
        let recognizer = CastleRecognizer::new();
        let mut positions = Vec::new();

        // Create multiple different positions
        for row in 6..9 {
            for col in 2..7 {
                let (board, king_pos) = create_mino_castle_board(Player::Black, row, col);
                positions.push((board, king_pos));
            }
        }

        b.iter(|| {
            for (board, king_pos) in &positions {
                recognizer.evaluate_castle(black_box(board), Player::Black, black_box(*king_pos));
            }
        });
    });

    // Benchmark cache eviction with small cache size
    group.bench_function("cache_eviction_small_cache", |b| {
        let recognizer = CastleRecognizer::with_cache_size(10);
        let mut positions = Vec::new();

        // Create many positions to force evictions
        for i in 0..50 {
            let row = 6 + (i % 3) as u8;
            let col = 2 + (i % 5) as u8;
            let (board, king_pos) = create_mino_castle_board(Player::Black, row, col);
            positions.push((board, king_pos));
        }

        b.iter(|| {
            for (board, king_pos) in &positions {
                recognizer.evaluate_castle(black_box(board), Player::Black, black_box(*king_pos));
            }
        });
    });

    // Benchmark different castle types
    group.bench_function("mino_castle_recognition", |b| {
        let recognizer = CastleRecognizer::new();
        let (board, king_pos) = create_mino_castle_board(Player::Black, 8, 4);

        b.iter(|| {
            recognizer.evaluate_castle(black_box(&board), Player::Black, black_box(king_pos));
        });
    });

    group.bench_function("anaguma_castle_recognition", |b| {
        let recognizer = CastleRecognizer::new();
        let (board, king_pos) = create_anaguma_castle_board(Player::Black, 8, 2);

        b.iter(|| {
            recognizer.evaluate_castle(black_box(&board), Player::Black, black_box(king_pos));
        });
    });

    // Benchmark cache statistics overhead
    group.bench_function("cache_stats_overhead", |b| {
        let recognizer = CastleRecognizer::new();
        let (board, king_pos) = create_mino_castle_board(Player::Black, 8, 4);

        // Warm up cache
        recognizer.evaluate_castle(&board, Player::Black, king_pos);

        b.iter(|| {
            recognizer.evaluate_castle(black_box(&board), Player::Black, black_box(king_pos));
            black_box(recognizer.get_cache_stats());
        });
    });

    // Benchmark with symmetry enabled
    group.bench_function("symmetry_enabled", |b| {
        let mut recognizer = CastleRecognizer::new();
        recognizer.set_symmetry_enabled(true);

        // Create mirrored positions
        let (board1, king_pos1) = create_mino_castle_board(Player::Black, 8, 2);
        let (board2, king_pos2) = create_mino_castle_board(Player::Black, 8, 6);

        b.iter(|| {
            recognizer.evaluate_castle(black_box(&board1), Player::Black, black_box(king_pos1));
            recognizer.evaluate_castle(black_box(&board2), Player::Black, black_box(king_pos2));
        });
    });

    // Benchmark cache size impact
    for cache_size in [50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("cache_size_impact", cache_size),
            cache_size,
            |b, &size| {
                let recognizer = CastleRecognizer::with_cache_size(size);
                let mut positions = Vec::new();

                // Create positions
                for i in 0..100 {
                    let row = 6 + (i % 3) as u8;
                    let col = 2 + (i % 7) as u8;
                    let (board, king_pos) = create_mino_castle_board(Player::Black, row, col);
                    positions.push((board, king_pos));
                }

                b.iter(|| {
                    for (board, king_pos) in &positions {
                        recognizer.evaluate_castle(
                            black_box(board),
                            Player::Black,
                            black_box(*king_pos),
                        );
                    }
                });
            },
        );
    }

    // Benchmark using fixtures
    group.bench_function("fixtures_throughput", |b| {
        let recognizer = CastleRecognizer::new();
        let fixtures = castle_fixtures();

        b.iter(|| {
            for fixture in &fixtures {
                let (board, king_pos) = (fixture.builder)(fixture.player);
                black_box(recognizer.evaluate_castle(black_box(&board), fixture.player, king_pos));
            }
        });
    });

    // Benchmark telemetry overhead
    group.bench_function("telemetry_overhead", |b| {
        let evaluator = KingSafetyEvaluator::new();
        let fixtures = castle_fixtures();

        b.iter(|| {
            for fixture in &fixtures {
                let (board, _) = (fixture.builder)(fixture.player);
                evaluator.evaluate(black_box(&board), fixture.player);
                black_box(evaluator.stats());
            }
        });
    });

    // Benchmark across game phases (opening/middlegame/endgame)
    group.bench_function("opening_phase_castles", |b| {
        let recognizer = CastleRecognizer::new();
        let fixtures = castle_fixtures();
        let opening_fixtures: Vec<_> = fixtures
            .iter()
            .filter(|f| {
                f.theme == shogi_engine::evaluation::castle_fixtures::CastleFixtureTheme::Canonical
            })
            .collect();

        b.iter(|| {
            for fixture in &opening_fixtures {
                let (board, king_pos) = (fixture.builder)(fixture.player);
                black_box(recognizer.evaluate_castle(black_box(&board), fixture.player, king_pos));
            }
        });
    });

    group.bench_function("middlegame_phase_castles", |b| {
        let recognizer = CastleRecognizer::new();
        let fixtures = castle_fixtures();
        let middlegame_fixtures: Vec<_> = fixtures
            .iter()
            .filter(|f| {
                f.theme == shogi_engine::evaluation::castle_fixtures::CastleFixtureTheme::Partial
                    || f.theme
                        == shogi_engine::evaluation::castle_fixtures::CastleFixtureTheme::Attacked
            })
            .collect();

        b.iter(|| {
            for fixture in &middlegame_fixtures {
                let (board, king_pos) = (fixture.builder)(fixture.player);
                black_box(recognizer.evaluate_castle(black_box(&board), fixture.player, king_pos));
            }
        });
    });

    group.bench_function("endgame_phase_castles", |b| {
        let recognizer = CastleRecognizer::new();
        let fixtures = castle_fixtures();
        let endgame_fixtures: Vec<_> = fixtures
            .iter()
            .filter(|f| {
                f.theme == shogi_engine::evaluation::castle_fixtures::CastleFixtureTheme::Broken
            })
            .collect();

        b.iter(|| {
            for fixture in &endgame_fixtures {
                let (board, king_pos) = (fixture.builder)(fixture.player);
                black_box(recognizer.evaluate_castle(black_box(&board), fixture.player, king_pos));
            }
        });
    });

    group.finish();
}

criterion_group!(benches, castle_recognition_cache_benchmark);
criterion_main!(benches);
