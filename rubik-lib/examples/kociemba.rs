use std::time::Duration;

use itertools::join;
use rubik_lib::{
    core::{moves::Move, scramble::scramble, state::State},
    solve::kociemba::{self},
};

fn check_sol(mut state: State, moves: &[u8]) {
    for &mv in moves {
        state = state * State::BASIC_MOVES[mv as usize];
    }
    assert_eq!(state, State::ID);
}

fn main() {
    let mut solver = kociemba::Solver::from_folder("data/kociemba").expect("failed to init solver");
    let n = 100;
    let (mvs, state) = scramble(n);
    println!("scramble ({} moves): {}", n, join(mvs, " "));
    let sol = solver.solve_timeout(&state, Duration::from_millis(1000));
    check_sol(state, &sol);
    println!("solution ({} moves): {}", sol.len(), join(sol.iter().map(|&i| Move::BASIC_MOVES[i as usize]), " "));
}
