use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group};
use rubik_lib::{
    algebra::coord::{CO, CP, Coord, EO, LR},
    core::moves::Moves,
};

criterion_group!(coord, bench_from_state, bench_to_state);

pub fn bench_from_state(c: &mut Criterion) {
    fn bench<'a, C: Coord>(c: &mut Criterion, name: &str) {
        let states: Vec<_> = Moves::to_depth(4).map(|(_, s)| s).collect();
        let mut g = c.benchmark_group(format!("coord/{name}"));
        g.throughput(Throughput::Elements(states.len() as u64));
        g.bench_function("from_state", |b| {
            b.iter(|| {
                for s in &states {
                    black_box(C::from_state(s));
                }
            });
        });
    }

    bench::<CO>(c, "CO");
    bench::<EO>(c, "EO");
    bench::<LR>(c, "LR");
    bench::<CP>(c, "CP");
}

pub fn bench_to_state(c: &mut Criterion) {
    fn bench<'a, C: Coord>(c: &mut Criterion, name: &str) {
        let mut g = c.benchmark_group(format!("coord/{name}"));
        g.throughput(Throughput::Elements(C::raw_to_usize(C::NUM_RAW) as u64));
        g.bench_function("to_state", |b| {
            b.iter(|| {
                for i in C::raw_coords() {
                    black_box(C::from_coord(i).sample_state());
                }
            });
        });
    }

    bench::<CO>(c, "CO");
    bench::<EO>(c, "EO");
    bench::<LR>(c, "LR");
    bench::<CP>(c, "CP");
}
