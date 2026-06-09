use criterion::criterion_main;
mod coord;
mod kociemba;
mod product;
mod state;
mod trie;

use coord::coord;
use kociemba::kociemba;
use product::product;
use state::state;
use trie::trie;

criterion_main!(state, trie, product, coord, kociemba);
