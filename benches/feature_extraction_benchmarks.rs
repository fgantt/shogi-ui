use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shogi_engine::tuning::feature_extractor::FeatureExtractor;
use shogi_engine::types::{CapturedPieces, Player};
use shogi_engine::BitboardBoard;

fn benchmark_mobility_feature_extraction(c: &mut Criterion) {
    let extractor = FeatureExtractor::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    c.bench_function("extract_mobility_features", |b| {
        b.iter(|| {
            let _features = black_box(extractor.extract_mobility_features(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

fn benchmark_coordination_feature_extraction(c: &mut Criterion) {
    let extractor = FeatureExtractor::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    c.bench_function("extract_coordination_features", |b| {
        b.iter(|| {
            let _features = black_box(extractor.extract_coordination_features(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

fn benchmark_full_feature_extraction(c: &mut Criterion) {
    let extractor = FeatureExtractor::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    c.bench_function("extract_features_full", |b| {
        b.iter(|| {
            let _features = black_box(extractor.extract_features(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });
}

fn benchmark_mobility_vs_heuristic(c: &mut Criterion) {
    let extractor = FeatureExtractor::new();
    let board = BitboardBoard::new();
    let captured_pieces = CapturedPieces::new();

    let mut group = c.benchmark_group("mobility_comparison");
    
    group.bench_function("actual_move_generation", |b| {
        b.iter(|| {
            let _features = black_box(extractor.extract_mobility_features(
                black_box(&board),
                black_box(Player::Black),
                black_box(&captured_pieces),
            ));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_mobility_feature_extraction,
    benchmark_coordination_feature_extraction,
    benchmark_full_feature_extraction,
    benchmark_mobility_vs_heuristic
);
criterion_main!(benches);

