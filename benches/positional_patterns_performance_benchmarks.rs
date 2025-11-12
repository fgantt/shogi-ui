use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::evaluation::positional_fixtures::{positional_fixtures, PositionalFixture};
use shogi_engine::evaluation::positional_patterns::PositionalPatternAnalyzer;
use shogi_engine::types::{CapturedPieces, Player};

fn positional_patterns_benchmark(c: &mut Criterion) {
    let fixtures = positional_fixtures();
    let mut group = c.benchmark_group("positional_patterns");

    for fixture in fixtures {
        let (board, captured) = (fixture.builder)();
        bench_fixture(&mut group, &fixture, board.clone(), captured.clone());
    }

    group.finish();
}

fn bench_fixture(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    fixture: &PositionalFixture,
    board: shogi_engine::bitboards::BitboardBoard,
    captured: CapturedPieces,
) {
    group.bench_with_input(
        BenchmarkId::new("fresh_analyzer", fixture.name),
        fixture.name,
        |b, _| {
            b.iter(|| {
                let mut analyzer = PositionalPatternAnalyzer::new();
                analyzer.evaluate_position(black_box(&board), Player::Black, black_box(&captured));
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("reused_analyzer", fixture.name),
        fixture.name,
        |b, _| {
            let mut analyzer = PositionalPatternAnalyzer::new();
            b.iter(|| {
                analyzer.evaluate_position(black_box(&board), Player::Black, black_box(&captured));
                analyzer.reset_stats();
            });
        },
    );
}

criterion_group!(benches, positional_patterns_benchmark);
criterion_main!(benches);
