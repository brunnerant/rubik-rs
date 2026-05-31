use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group};
use itertools::Itertools;
use rubik_lib::{
    model::{moves::Moves, state::State},
    solve::four_list::product::Product,
};

criterion_group!(product, bench_product);

fn states_to_depth(d: u8) -> Vec<State> {
    Moves::to_depth(d).map(|(_, s)| s).unique().collect()
}

pub fn bench_product(c: &mut Criterion) {
    let h = 2;
    let half: Vec<_> = states_to_depth(h);
    let mut sorted_full: Vec<_> = states_to_depth(2 * h);
    sorted_full.sort();
    let product = Product::sorted(half.iter().cloned(), half.iter().cloned());

    let mut g = c.benchmark_group("product");
    g.throughput(Throughput::Elements(sorted_full.len() as u64));
    g.bench_function("iter", |b| {
        b.iter(|| {
            for x in product.clone() {
                black_box(x);
            }
        })
    });
}
