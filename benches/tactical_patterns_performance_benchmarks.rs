//! Tactical pattern recognition performance benchmarks
//!
//! Measures:
//! - Recognizer construction and configuration updates
//! - Single-position evaluation for representative tactical motifs
//! - Batch evaluation throughput across the tactical corpus

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde::Deserialize;
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::evaluation::tactical_patterns::{TacticalConfig, TacticalPatternRecognizer};
use shogi_engine::types::{CapturedPieces, Player};

#[derive(Debug, Deserialize)]
struct TacticalCorpus {
    positions: Vec<TacticalPosition>,
}

#[derive(Debug, Deserialize)]
struct TacticalPosition {
    name: String,
    fen: String,
    expectation: String,
}

#[derive(Clone)]
struct PreparedPosition {
    name: String,
    board: BitboardBoard,
    player: Player,
    captured: CapturedPieces,
}

fn load_corpus() -> Vec<PreparedPosition> {
    let corpus: TacticalCorpus = toml::from_str(include_str!("../tests/data/tactical_corpus.toml"))
        .expect("valid tactical corpus");

    corpus
        .positions
        .into_iter()
        .map(|position| {
            let (board, player, captured) =
                BitboardBoard::from_fen(&position.fen).unwrap_or_else(|_| {
                    panic!("invalid tactical FEN {} ({})", position.fen, position.name)
                });
            PreparedPosition {
                name: position.name,
                board,
                player,
                captured,
            }
        })
        .collect()
}

fn benchmark_recognizer_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("tactical_recognizer_construction");

    group.bench_function("new", |b| {
        b.iter(|| black_box(TacticalPatternRecognizer::new()));
    });

    group.bench_function("with_config", |b| {
        b.iter(|| {
            black_box(TacticalPatternRecognizer::with_config(
                TacticalConfig::default(),
            ));
        });
    });

    group.bench_function("set_config", |b| {
        let mut recognizer = TacticalPatternRecognizer::new();
        b.iter(|| {
            recognizer.set_config(black_box(TacticalConfig::aggressive()));
        });
    });

    group.finish();
}

fn benchmark_corpus_evaluation(c: &mut Criterion) {
    let positions = load_corpus();
    let mut group = c.benchmark_group("tactical_single_position");

    for position in &positions {
        group.bench_with_input(
            BenchmarkId::new("evaluate", &position.name),
            position,
            |b, case| {
                let mut recognizer =
                    TacticalPatternRecognizer::with_config(TacticalConfig::default());
                b.iter(|| {
                    black_box(recognizer.evaluate_tactics(
                        &case.board,
                        case.player,
                        &case.captured,
                    ));
                });
            },
        );
    }

    group.finish();
}

fn benchmark_corpus_batch(c: &mut Criterion) {
    let positions = load_corpus();
    let mut group = c.benchmark_group("tactical_batch_evaluation");

    group.bench_function("evaluate_all_once", |b| {
        b.iter(|| {
            let mut recognizer = TacticalPatternRecognizer::with_config(TacticalConfig::default());
            for case in &positions {
                black_box(recognizer.evaluate_tactics(&case.board, case.player, &case.captured));
            }
        });
    });

    group.bench_function("evaluate_all_16x", |b| {
        b.iter(|| {
            let mut recognizer = TacticalPatternRecognizer::with_config(TacticalConfig::default());
            for _ in 0..16 {
                for case in &positions {
                    black_box(recognizer.evaluate_tactics(
                        &case.board,
                        case.player,
                        &case.captured,
                    ));
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    tactical_patterns,
    benchmark_recognizer_construction,
    benchmark_corpus_evaluation,
    benchmark_corpus_batch
);
criterion_main!(tactical_patterns);
