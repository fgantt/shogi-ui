//! Performance benchmarks for adaptive tuning
//!
//! This benchmark suite compares static vs adaptive tuning:
//! - Static tuning: Fixed LMR parameters
//! - Adaptive tuning: Parameters adjusted dynamically based on performance
//!
//! Metrics measured:
//! - LMR effectiveness improvement
//! - Parameter adjustment frequency
//! - Tuning stability (no oscillation)
//! - Search time impact
//! - Tuning effectiveness
//!
//! Expected results:
//! - Adaptive tuning should improve LMR effectiveness
//! - Tuning should not cause oscillation or instability
//! - Overhead should be minimal (<1% search time)
//! - Parameters should stabilize after initial adjustments

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shogi_engine::{
    bitboards::BitboardBoard,
    search::SearchEngine,
    types::{
        AdaptiveTuningConfig, CapturedPieces, GamePhase, LMRConfig, Player, PositionClassification,
        TuningAggressiveness,
    },
};
use std::time::Duration;

/// Create a test engine with static tuning (no adaptive tuning)
fn create_test_engine_static() -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16);
    let mut config = LMRConfig::default();
    config.adaptive_tuning_config.enabled = false;
    engine.update_lmr_config(config).unwrap();
    engine
}

/// Create a test engine with adaptive tuning
fn create_test_engine_adaptive(aggressiveness: TuningAggressiveness) -> SearchEngine {
    let mut engine = SearchEngine::new(None, 16);
    let mut config = LMRConfig::default();
    config.adaptive_tuning_config.enabled = true;
    config.adaptive_tuning_config.aggressiveness = aggressiveness;
    config.adaptive_tuning_config.min_data_threshold = 100;
    engine.update_lmr_config(config).unwrap();
    engine
}

/// Benchmark static vs adaptive tuning
fn benchmark_static_vs_adaptive(c: &mut Criterion) {
    let mut group = c.benchmark_group("static_vs_adaptive");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    // Test across different depths
    for depth in [3, 4, 5, 6] {
        group.bench_with_input(BenchmarkId::new("depth", depth), &depth, |b, &depth| {
            b.iter(|| {
                let mut engine_static = create_test_engine_static();
                let mut engine_adaptive =
                    create_test_engine_adaptive(TuningAggressiveness::Moderate);

                engine_static.reset_lmr_stats();
                engine_adaptive.reset_lmr_stats();

                let mut board_mut = board.clone();
                let result_static = engine_static.search_at_depth_legacy(
                    black_box(&mut board_mut),
                    black_box(&captured_pieces),
                    player,
                    depth,
                    1000,
                );

                let mut board_mut = board.clone();
                let result_adaptive = engine_adaptive.search_at_depth_legacy(
                    black_box(&mut board_mut),
                    black_box(&captured_pieces),
                    player,
                    depth,
                    1000,
                );

                // Perform adaptive tuning after search
                let _tuning_result = engine_adaptive.auto_tune_lmr_parameters(
                    Some(GamePhase::Middlegame),
                    Some(PositionClassification::Neutral),
                );

                let stats_static = engine_static.get_lmr_stats().clone();
                let stats_adaptive = engine_adaptive.get_lmr_stats().clone();

                black_box((result_static, result_adaptive, stats_static, stats_adaptive))
            });
        });
    }

    group.finish();
}

/// Benchmark different tuning aggressiveness levels
fn benchmark_tuning_aggressiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("tuning_aggressiveness");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    let aggressiveness_levels = vec![
        ("conservative", TuningAggressiveness::Conservative),
        ("moderate", TuningAggressiveness::Moderate),
        ("aggressive", TuningAggressiveness::Aggressive),
    ];

    for (name, aggressiveness) in aggressiveness_levels {
        group.bench_with_input(
            BenchmarkId::new("aggressiveness", name),
            &aggressiveness,
            |b, _aggressiveness| {
                b.iter(|| {
                    let mut engine = create_test_engine_adaptive(*_aggressiveness);
                    engine.reset_lmr_stats();

                    let mut board_mut = board.clone();
                    let result = engine.search_at_depth_legacy(
                        black_box(&mut board_mut),
                        black_box(&captured_pieces),
                        player,
                        5, // Fixed depth
                        1000,
                    );

                    // Perform adaptive tuning after search
                    let _tuning_result = engine.auto_tune_lmr_parameters(
                        Some(GamePhase::Middlegame),
                        Some(PositionClassification::Neutral),
                    );

                    let stats = engine.get_lmr_stats().clone();
                    let tuning_stats = stats.adaptive_tuning_stats.clone();

                    black_box((result, stats, tuning_stats))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark tuning effectiveness
fn benchmark_tuning_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("tuning_effectiveness");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("effectiveness_measurement", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_adaptive(TuningAggressiveness::Moderate);
            engine.reset_lmr_stats();

            let mut board_mut = board.clone();
            let _result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            // Perform adaptive tuning
            let _tuning_result = engine.auto_tune_lmr_parameters(
                Some(GamePhase::Middlegame),
                Some(PositionClassification::Neutral),
            );

            let stats = engine.get_lmr_stats().clone();
            let tuning_stats = stats.adaptive_tuning_stats.clone();

            // Calculate effectiveness metrics
            let efficiency = stats.efficiency();
            let research_rate = stats.research_rate();
            let cutoff_rate = stats.cutoff_rate();
            let tuning_success_rate = tuning_stats.success_rate();
            let parameter_changes = tuning_stats.parameter_changes;

            black_box((
                stats,
                tuning_stats,
                efficiency,
                research_rate,
                cutoff_rate,
                tuning_success_rate,
                parameter_changes,
            ))
        });
    });

    group.finish();
}

/// Benchmark tuning stability (no oscillation)
fn benchmark_tuning_stability(c: &mut Criterion) {
    let mut group = c.benchmark_group("tuning_stability");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("stability_measurement", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_adaptive(TuningAggressiveness::Moderate);
            engine.reset_lmr_stats();

            // Track parameter changes over multiple tuning cycles
            let mut parameter_history = Vec::new();

            for _cycle in 0..5 {
                let mut board_mut = board.clone();
                let _result = engine.search_at_depth_legacy(
                    black_box(&mut board_mut),
                    black_box(&captured_pieces),
                    player,
                    5, // Fixed depth
                    1000,
                );

                // Record current parameters
                let config = engine.get_lmr_config();
                parameter_history.push((
                    config.base_reduction,
                    config.max_reduction,
                    config.min_move_index,
                ));

                // Perform adaptive tuning
                let _tuning_result = engine.auto_tune_lmr_parameters(
                    Some(GamePhase::Middlegame),
                    Some(PositionClassification::Neutral),
                );
            }

            let stats = engine.get_lmr_stats().clone();
            let tuning_stats = stats.adaptive_tuning_stats.clone();

            // Check for oscillation (parameters should stabilize)
            let oscillation_detected = parameter_history.windows(3).any(|w| {
                w[0] == w[2] && w[0] != w[1] // Parameters oscillating
            });

            black_box((stats, tuning_stats, parameter_history, oscillation_detected))
        });
    });

    group.finish();
}

/// Benchmark game phase-based tuning
fn benchmark_game_phase_tuning(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_phase_tuning");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    let game_phases = vec![
        ("opening", GamePhase::Opening),
        ("middlegame", GamePhase::Middlegame),
        ("endgame", GamePhase::Endgame),
    ];

    for (name, phase) in game_phases {
        group.bench_with_input(BenchmarkId::new("phase", name), &phase, |b, _phase| {
            b.iter(|| {
                let mut engine = create_test_engine_adaptive(TuningAggressiveness::Moderate);
                engine.reset_lmr_stats();

                // Set up stats with sufficient data
                engine.lmr_stats.moves_considered = 100;
                engine.lmr_stats.reductions_applied = 50;

                let result = engine
                    .auto_tune_lmr_parameters(Some(*_phase), Some(PositionClassification::Neutral));

                let stats = engine.get_lmr_stats().clone();
                let tuning_stats = stats.adaptive_tuning_stats.clone();

                black_box((result, stats, tuning_stats))
            });
        });
    }

    group.finish();
}

/// Benchmark position type-based tuning
fn benchmark_position_type_tuning(c: &mut Criterion) {
    let mut group = c.benchmark_group("position_type_tuning");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    let position_types = vec![
        ("tactical", PositionClassification::Tactical),
        ("quiet", PositionClassification::Quiet),
        ("neutral", PositionClassification::Neutral),
    ];

    for (name, position_type) in position_types {
        group.bench_with_input(
            BenchmarkId::new("position_type", name),
            &position_type,
            |b, _position_type| {
                b.iter(|| {
                    let mut engine = create_test_engine_adaptive(TuningAggressiveness::Moderate);
                    engine.reset_lmr_stats();

                    // Set up stats with sufficient data
                    engine.lmr_stats.moves_considered = 100;
                    engine.lmr_stats.reductions_applied = 50;

                    let result = engine.auto_tune_lmr_parameters(
                        Some(GamePhase::Middlegame),
                        Some(*_position_type),
                    );

                    let stats = engine.get_lmr_stats().clone();
                    let tuning_stats = stats.adaptive_tuning_stats.clone();

                    black_box((result, stats, tuning_stats))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark comprehensive tuning analysis
fn benchmark_comprehensive_tuning_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_tuning_analysis");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();
    let player = Player::Black;

    group.bench_function("comprehensive_analysis", |b| {
        b.iter(|| {
            let mut engine = create_test_engine_adaptive(TuningAggressiveness::Moderate);
            engine.reset_lmr_stats();

            let mut board_mut = board.clone();
            let start = std::time::Instant::now();

            let result = engine.search_at_depth_legacy(
                black_box(&mut board_mut),
                black_box(&captured_pieces),
                player,
                5, // Fixed depth
                1000,
            );

            let elapsed = start.elapsed();

            // Perform adaptive tuning
            let tuning_result = engine.auto_tune_lmr_parameters(
                Some(GamePhase::Middlegame),
                Some(PositionClassification::Neutral),
            );

            let stats = engine.get_lmr_stats().clone();
            let tuning_stats = stats.adaptive_tuning_stats.clone();

            // Comprehensive metrics
            let efficiency = stats.efficiency();
            let research_rate = stats.research_rate();
            let cutoff_rate = stats.cutoff_rate();
            let tuning_success_rate = tuning_stats.success_rate();
            let parameter_changes = tuning_stats.parameter_changes;
            let game_phase_adjustments = tuning_stats.game_phase_adjustments;
            let position_type_adjustments = tuning_stats.position_type_adjustments;

            black_box((
                result,
                elapsed,
                tuning_result,
                stats,
                tuning_stats,
                efficiency,
                research_rate,
                cutoff_rate,
                tuning_success_rate,
                parameter_changes,
                game_phase_adjustments,
                position_type_adjustments,
            ))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_static_vs_adaptive,
    benchmark_tuning_aggressiveness,
    benchmark_tuning_effectiveness,
    benchmark_tuning_stability,
    benchmark_game_phase_tuning,
    benchmark_position_type_tuning,
    benchmark_comprehensive_tuning_analysis
);

criterion_main!(benches);
