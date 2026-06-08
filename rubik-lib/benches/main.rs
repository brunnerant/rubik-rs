use criterion::criterion_main;
mod product;
mod state;
mod trie;
mod coord;

use product::product;
use state::state;
use trie::trie;
use coord::coord;

criterion_main!(state, trie, product, coord);
