use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, SamplingMode};
use shogi_engine::bitboards::BitboardBoard;
use shogi_engine::types::{CapturedPieces, Player};
use shogi_engine::moves::MoveGenerator;

fn bench_board_cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("board_clone");
    group.sampling_mode(SamplingMode::Auto);

    let board = BitboardBoard::new();
    let captured = CapturedPieces::new();
    let player = Player::Black;
    let mg = MoveGenerator::new();
    let legal = mg.generate_legal_moves(&board, player, &captured);

    group.bench_with_input(BenchmarkId::new("BitboardBoard::clone", legal.len()), &legal.len(), |b, _| {
        b.iter(|| {
            let _b2 = board.clone();
        });
    });

    group.bench_function("CapturedPieces::clone", |b| {
        b.iter(|| {
            let _c2 = captured.clone();
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
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_board_cloning);
criterion_main!(benches);


