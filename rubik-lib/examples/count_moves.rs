use std::{
    collections::{HashSet, VecDeque},
    env::args,
};

use itertools::Itertools;
use rubik_lib::core::{moves::Moves, state::State, sym::Symmetries};

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

    let sym = Symmetries::all();
    let mut visited = HashSet::new();
    let mut to_visit = VecDeque::from([(State::ID, 0)]);
    while let Some((state, d)) = to_visit.pop_front() {
        visited.insert(state);
        if d >= depth {
            continue;
        }
        for mv in State::BASIC_MOVES {
            let next_state = mv * state;
            let (repr, _) = sym.repr(next_state);
            if !visited.contains(&repr) {
                to_visit.push_back((repr, d + 1));
            }
        }
    }

    println!(
        "This reduces to {} unique moves when factoring out symmetries",
        visited.len()
    );
}
