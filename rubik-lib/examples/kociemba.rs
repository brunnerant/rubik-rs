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

    let mut total_length = 0;
    for _ in 0..n {
        let (_, state) = scramble(100);
        let sol = solver.solve_timeout(&state, Duration::from_millis(10));
        check_sol(state, &sol);
        total_length += sol.len();
        let mvs: Vec<_> = sol
            .into_iter()
            .map(|i| Move::BASIC_MOVES[i as usize])
            .collect();
        println!("len {}: {}", mvs.len(), join(mvs, " "));
    }
    println!("avg len: {:.01}", total_length as f32 / n as f32)
}
