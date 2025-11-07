use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::moves::MoveGenerator;
use shogi_engine::types::{CapturedPieces, Player};

/// Benchmark debug logging overhead
/// Compares performance with verbose-debug feature enabled vs disabled
fn bench_debug_logging_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("debug_logging");

    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let mg = MoveGenerator::new();
    let legal_moves = mg.generate_legal_moves(&board, player, &captured);

    // Benchmark trace_log calls with format! strings (typical usage)
    group.bench_function("trace_log_with_format_disabled", |b| {
        // Simulate what happens when verbose-debug feature is disabled
        // The function should return early without string formatting
        b.iter(|| {
            for _ in 0..100 {
                crate::debug_utils::trace_log("BENCH", "Test message");
            }
        });
    });

    // Benchmark format! string creation (overhead even when disabled)
    group.bench_function("format_string_creation", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _msg = format!("Test message: depth={} alpha={} beta={}", 5, -100, 100);
            }
        });
    });

    // Benchmark decision logging overhead
    group.bench_function("log_decision_disabled", |b| {
        b.iter(|| {
            for _ in 0..100 {
                crate::debug_utils::log_decision("BENCH", "Test", "Reason", Some(42));
            }
        });
    });

    // Benchmark move evaluation logging overhead
    group.bench_function("log_move_eval_disabled", |b| {
        if let Some(mv) = legal_moves.first() {
            b.iter(|| {
                for _ in 0..100 {
                    crate::debug_utils::log_move_eval("BENCH", &mv.to_usi_string(), 42, "test");
                }
            });
        }
    });

    // Benchmark with format! calls (simulates real usage)
    group.bench_function("trace_log_format_simulation", |b| {
        let depth = 5;
        let alpha = -100;
        let beta = 100;
        b.iter(|| {
            for _ in 0..100 {
                // Simulate typical call: trace_log("FEATURE", &format!(...))
                let msg = format!("depth={} alpha={} beta={}", depth, alpha, beta);
                crate::debug_utils::trace_log("BENCH", &msg);
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_debug_logging_overhead);
criterion_main!(benches);
