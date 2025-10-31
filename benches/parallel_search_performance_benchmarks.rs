use criterion::{criterion_group, criterion_main, Criterion, Throughput, SamplingMode, BenchmarkId};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::types::{CapturedPieces, Player};
use shogi_engine::moves::MoveGenerator;
use shogi_engine::search::search_engine::{SearchEngine, IterativeDeepening};

fn bench_root_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_root_search");
    group.sampling_mode(SamplingMode::Auto);

    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let mg = MoveGenerator::new();
    let legal = mg.generate_legal_moves(&board, player, &captured);
    group.throughput(Throughput::Elements(legal.len() as u64));

    // Test across depths and thread counts
    for &depth in &[3u8, 5u8, 6u8] {
        for &threads in &[1usize, 2, 4, 8] {
            group.bench_with_input(BenchmarkId::new(format!("depth{}", depth), threads), &threads, |b, &t| {
                b.iter(|| {
                    // New engine per iteration to avoid cross-benchmark state
                    let mut engine = SearchEngine::new(None, 16);
                    let mut id = if t > 1 {
                        IterativeDeepening::new_with_threads(depth, 1000, None, t)
                    } else {
                        IterativeDeepening::new(depth, 1000, None)
                    };
                    let _ = id.search(&mut engine, &board, &captured, player);
                });
            });
        }
    }

    group.finish();
}

criterion_group!(benches, bench_root_search);
criterion_main!(benches);


