//! Benchmarks for strength testing performance
//!
//! These benchmarks measure the performance of strength testing with actual games
//! vs. the old simulation approach.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::tuning::validator::{MockGamePlayer, ShogiEngineGamePlayer, StrengthTester};
use shogi_engine::tuning::types::GameResult as TuningGameResult;
use shogi_engine::types::NUM_EVAL_FEATURES;

fn benchmark_strength_testing_with_mock(c: &mut Criterion) {
    let original_weights = vec![1.0; NUM_EVAL_FEATURES];
    let tuned_weights = vec![1.1; NUM_EVAL_FEATURES];

    c.bench_function("strength_testing_mock_10_games", |b| {
        let mock_results = vec![
            TuningGameResult::BlackWin,
            TuningGameResult::WhiteWin,
            TuningGameResult::Draw,
        ];
        let mock_player = Box::new(MockGamePlayer::new(mock_results));
        let tester = StrengthTester::with_game_player(10, 1000, mock_player);

        b.iter(|| {
            black_box(tester.test_engine_strength(
                black_box(&original_weights),
                black_box(&tuned_weights),
            ))
        });
    });
}

fn benchmark_strength_testing_with_engine(c: &mut Criterion) {
    let original_weights = vec![1.0; NUM_EVAL_FEATURES];
    let tuned_weights = vec![1.0; NUM_EVAL_FEATURES]; // Same weights for fair comparison

    // Use very fast settings for benchmarking
    c.bench_function("strength_testing_engine_2_games", |b| {
        let tester = StrengthTester::new(2, 50); // 2 games, 50ms per move

        b.iter(|| {
            black_box(tester.test_engine_strength(
                black_box(&original_weights),
                black_box(&tuned_weights),
            ))
        });
    });
}

fn benchmark_game_player_play_game(c: &mut Criterion) {
    let original_weights = vec![1.0; NUM_EVAL_FEATURES];
    let tuned_weights = vec![1.1; NUM_EVAL_FEATURES];

    c.bench_function("game_player_mock_play_game", |b| {
        let mock_results = vec![TuningGameResult::Draw];
        let mock_player = MockGamePlayer::new(mock_results);

        b.iter(|| {
            black_box(mock_player.play_game(
                black_box(&original_weights),
                black_box(&tuned_weights),
                black_box(1000),
                black_box(200),
            ))
        });
    });

    c.bench_function("game_player_engine_play_game", |b| {
        let engine_player = ShogiEngineGamePlayer::new(2, false); // Depth 2, not verbose

        b.iter(|| {
            black_box(engine_player.play_game(
                black_box(&original_weights),
                black_box(&tuned_weights),
                black_box(50), // Very fast for benchmarking
                black_box(50), // Short game limit
            ))
        });
    });
}

criterion_group!(
    benches,
    benchmark_strength_testing_with_mock,
    benchmark_strength_testing_with_engine,
    benchmark_game_player_play_game
);
criterion_main!(benches);

