use std::env::args;

use itertools::Itertools;
use rubik_lib::algo::moves::Moves;

fn main() {
    const DEFAULT_DEPTH: u8 = 5;
    let depth = args()
        .nth(1)
        .map(|arg| arg.parse::<u8>().unwrap_or(DEFAULT_DEPTH))
        .unwrap_or(DEFAULT_DEPTH);

    println!(
        "There are {} unique moves to depth {}",
        Moves::to_depth(depth).unique_by(|&(_, s)| s).count(),
        depth
    );
}
