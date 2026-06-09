use criterion::criterion_main;
mod coord;
mod product;
mod state;
mod trie;
mod kociemba;

use coord::coord;
use product::product;
use state::state;
use trie::trie;
use kociemba::kociemba;

criterion_main!(state, trie, product, coord, kociemba);
