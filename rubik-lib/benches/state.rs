use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group};
use itertools::iproduct;
use rubik_lib::model::state::State;

criterion_group!(state, bench_mul, bench_inv);

pub fn bench_mul(c: &mut Criterion) {
    let mut g = c.benchmark_group("state");
    g.throughput(Throughput::Elements(18 * 18));
    g.bench_function("mul", |b| {
        b.iter(|| {
            for (a, b) in iproduct!(0..18, 0..18) {
                black_box(State::BASIC_MOVES[a] * State::BASIC_MOVES[b]);
            }
        })
    });
}

pub fn bench_inv(c: &mut Criterion) {
    let mut g = c.benchmark_group("state");
    g.throughput(Throughput::Elements(18));
    g.bench_function("inv", |b| {
        b.iter(|| {
            for s in State::BASIC_MOVES {
                black_box(s.inv());
            }
        })
    });
}
