use std::{collections::HashSet, env::args};

use rubik_lib::{moves::Move, state::State};

fn main() {
    const DEFAULT_DEPTH: usize = 5;
    let depth = args()
        .nth(1)
        .map(|arg| arg.parse::<usize>().unwrap_or(DEFAULT_DEPTH))
        .unwrap_or(DEFAULT_DEPTH);

    let mut states_to_check = vec![State::SOLVED];
    let mut known_states = HashSet::from([State::SOLVED]);
    let basic_moves = Move::BASIC_MOVES.map(|m| State::SOLVED.mv(m));
    for _ in 0..depth {
        let mut next_states_to_check = vec![];
        for state in states_to_check.drain(..) {
            for mv in basic_moves {
                let next_state = state.compose(&mv);
                if !known_states.contains(&next_state) {
                    known_states.insert(next_state);
                    next_states_to_check.push(next_state);
                }
            }
        }
        states_to_check.append(&mut next_states_to_check);
    }

    println!(
        "There are {} unique moves to depth {}",
        known_states.len(),
        depth
    );
}
