use criterion::criterion_main;
mod state;
mod trie;

use state::state;
use trie::trie;

criterion_main!(state, trie);
