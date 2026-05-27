use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group};
use itertools::iproduct;
use rubik_lib::model::{moves::Move, state::State};

criterion_group!(state, bench_mv, bench_composition);

pub fn bench_mv(c: &mut Criterion) {
    let basic_moves = Move::BASIC_MOVES;
    let mut g = c.benchmark_group("state");
    g.throughput(Throughput::Elements(basic_moves.len() as u64));
    g.bench_function("mv", |b| {
        b.iter(|| {
            for m in basic_moves {
                black_box(State::SOLVED.mv(m));
            }
        })
    });
}

pub fn bench_composition(c: &mut Criterion) {
    let basic_states = Move::BASIC_MOVES.map(|m| State::SOLVED.mv(m));
    let mut g = c.benchmark_group("state");
    g.throughput(Throughput::Elements(
        (basic_states.len() * basic_states.len()) as u64,
    ));
    g.bench_function("compose", |b| {
        b.iter(|| {
            for (a, b) in iproduct!(basic_states, basic_states) {
                black_box(a.compose(&b));
            }
        })
    });
}
