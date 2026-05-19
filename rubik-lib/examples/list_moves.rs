use std::env::args;

use itertools::join;
use rubik_lib::util::Moves;

fn main() {
    const DEFAULT_DEPTH: usize = 5;
    let depth = args()
        .nth(1)
        .map(|arg| arg.parse::<usize>().unwrap_or(DEFAULT_DEPTH))
        .unwrap_or(DEFAULT_DEPTH);

    for (moves, _) in Moves::to_depth(depth).iter() {
        println!("{}", join(moves, " "));
    }
}
