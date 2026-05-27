use criterion::criterion_main;
mod product;
mod state;
mod trie;

use product::product;
use state::state;
use trie::trie;

criterion_main!(state, trie, product);
