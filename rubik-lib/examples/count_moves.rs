use std::env::args;

use rubik_lib::algo::moves::Moves;

fn main() {
    const DEFAULT_DEPTH: usize = 5;
    let depth = args()
        .nth(1)
        .map(|arg| arg.parse::<usize>().unwrap_or(DEFAULT_DEPTH))
        .unwrap_or(DEFAULT_DEPTH);

    println!(
        "There are {} unique moves to depth {}",
        Moves::to_depth(depth).count(),
        depth
    );
}
