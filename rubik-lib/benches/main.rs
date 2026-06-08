use criterion::criterion_main;
mod coord;
mod product;
mod state;
mod trie;

use coord::coord;
use product::product;
use state::state;
use trie::trie;

criterion_main!(state, trie, product, coord);
