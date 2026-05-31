use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group};
use itertools::iproduct;
use rubik_lib::model::{moves::Move, state::State};

criterion_group!(state, bench_mv, bench_then, bench_inv);

pub fn bench_mv(c: &mut Criterion) {
    let basic_moves = Move::BASIC_MOVES;
    let mut g = c.benchmark_group("state");
    g.throughput(Throughput::Elements(basic_moves.len() as u64));
    g.bench_function("mv", |b| {
        b.iter(|| {
            for m in basic_moves {
                black_box(State::ID.mv(m));
            }
        })
    });
}

pub fn bench_then(c: &mut Criterion) {
    let basic_states = Move::BASIC_MOVES.map(|m| State::ID.mv(m));
    let mut g = c.benchmark_group("state");
    g.throughput(Throughput::Elements(
        (basic_states.len() * basic_states.len()) as u64,
    ));
    g.bench_function("then", |b| {
        b.iter(|| {
            for (a, b) in iproduct!(basic_states, basic_states) {
                black_box(a.then(&b));
            }
        })
    });
}

pub fn bench_inv(c: &mut Criterion) {
    let basic_states = Move::BASIC_MOVES.map(|m| State::ID.mv(m));
    let mut g = c.benchmark_group("state");
    g.throughput(Throughput::Elements(basic_states.len() as u64));
    g.bench_function("inv", |b| {
        b.iter(|| {
            for s in basic_states {
                black_box(s.inv());
            }
        })
    });
}
