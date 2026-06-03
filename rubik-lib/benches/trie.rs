use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group};
use itertools::Itertools;
use rubik_lib::{core::moves::Moves, solve::four_list::trie::TrieBuilder};

criterion_group!(trie, bench_iter);

pub fn bench_iter(c: &mut Criterion) {
    let mut trie = TrieBuilder::new();
    let mut count = 0;
    for (_, s) in Moves::to_depth(4).unique_by(|&(_, s)| s) {
        trie.insert(s);
        count += 1;
    }
    let trie = trie.build();

    let mut g = c.benchmark_group("trie");
    g.throughput(Throughput::Elements(count));
    g.bench_function("iter", |b| {
        b.iter(|| {
            for s in trie.ordered() {
                black_box(s);
            }
        })
    });
}
