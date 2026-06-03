use std::env::args;

use itertools::{Itertools, join};
use rubik_lib::core::moves::Moves;

fn main() {
    const DEFAULT_DEPTH: u8 = 5;
    let depth = args()
        .nth(1)
        .map(|arg| arg.parse::<u8>().unwrap_or(DEFAULT_DEPTH))
        .unwrap_or(DEFAULT_DEPTH);

    for (moves, _) in Moves::to_depth(depth).unique_by(|&(_, s)| s) {
        println!("{}", join(moves.moves(), " "));
    }
}
