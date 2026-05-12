use std::collections::{HashMap, hash_map::Entry};

use itertools::join;
use rubik_lib::{moves::Move, state::State};

fn main() {
    const DEPTH: usize = 5;
    let mut canon = vec![vec![]];
    let mut states_to_check = vec![(0, State::SOLVED)];
    let mut state_to_canon = HashMap::from([(State::SOLVED, 0)]);
    for _ in 0..DEPTH {
        let mut next_states_to_check = vec![];
        for (idx, state) in states_to_check.drain(..) {
            for mv in Move::BASIC_MOVES {
                let next_state = state.mv(mv);
                if let Entry::Vacant(e) = state_to_canon.entry(next_state) {
                    e.insert(canon.len());
                    next_states_to_check.push((canon.len(), next_state));
                    let mut moves = canon[idx].clone();
                    moves.push(mv);
                    canon.push(moves);
                }
            }
        }
        states_to_check.append(&mut next_states_to_check);
    }

    for moves in canon {
        println!("{}", join(moves, " "));
    }
}
