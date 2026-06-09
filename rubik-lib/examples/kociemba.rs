use itertools::join;
use rubik_lib::{
    core::{moves::Move, scramble::scramble, state::State},
    solve::kociemba::{self},
};

fn check_moves(mut state: State, moves: &[Move]) {
    for &mv in moves {
        state = state * mv;
    }
    assert_eq!(state, State::ID);
}

fn main() {
    let mut solver = kociemba::Solver::from_folder("data/kociemba").expect("failed to init solver");
    let (mvs, state) = scramble(100);
    println!("scramble:\n{}", join(mvs, " "));
    println!("solutions:");
    solver.init(&state);

    while let Some(mvs) = solver.step() {
        let mvs: Vec<_> = mvs
            .into_iter()
            .map(|i| Move::BASIC_MOVES[i as usize])
            .collect();
        check_moves(state, &mvs);
        println!("{}", join(mvs, " "));
    }
}
