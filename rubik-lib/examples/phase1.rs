use itertools::join;
use rubik_lib::{
    algebra::coord::{CO, Coord, EOLR},
    core::{moves::Move, scramble::scramble, state::State},
    solve::kociemba::{self},
};

fn check_phase1(mut state: State, moves: &Vec<Move>, coords: &kociemba::Coords) {
    for &mv in moves {
        state = state * mv;
    }
    let (_, j) = EOLR::unpack_sym_coord(
        coords
            .eolr_coord
            .raw_to_sym(EOLR::from_state(&state).coord()),
    );
    state = coords.sym.conj_inv(state, j);
    assert_eq!(0, EOLR::from_state(&state).coord());
    assert_eq!(0, CO::from_state(&state).coord());
}

fn main() {
    let mut solver = kociemba::Solver::from_folder("data/kociemba").expect("failed to init solver");
    let (mvs, state) = scramble(100);
    println!("scramble: {}", join(mvs, " "));
    solver.init(&state);
    println!("generating phase 1 move sequences:");

    let mut prev_l = 100;
    let mut prev_n = 0;
    loop {
        let Some(mvs) = solver.phase1_step() else {
            break;
        };

        // check_phase1(state, &mvs, &solver.coords);
        if mvs.len() != prev_l {
            prev_l = mvs.len();
            if prev_n > 0 {
                println!(" done {prev_n}");
                prev_n = 0;
            }
            print!("length {prev_l}");
        }
        prev_n += 1;
        // println!("{}", join(mvs, " "));
    }
}
