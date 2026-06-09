use std::io::Write;

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

fn num_display(n: usize) -> String {
    if n < 1_000 {
        format!("{}", n)
    } else if n < 1_000_000 {
        format!("{}K", n / 1_000)
    } else  {
        format!("{}M", n / 1_000_000)
    }
}

fn main() {
    let mut solver = kociemba::Solver::from_folder("data/kociemba").expect("failed to init solver");
    let (mvs, state) = scramble(100);
    println!("scramble: {}", join(mvs, " "));
    solver.init(&state);
    println!("generating phase 1 move sequences:");

    let mut l = 100;
    let mut n = 0;
    loop {
        let Some(mvs) = solver.phase1_step() else {
            break;
        };

        check_phase1(state, &mvs, &solver.coords);
        if mvs.len() != l {
            if n > 0 {
                println!("\rlength {l}: {}", num_display(n));
                n = 0;
            }
            l = mvs.len();
        }
        n += 1;
        if n % 1_000_000 == 0 {
            print!("\rlength {l}: {}", num_display(n));
            let _ = std::io::stdout().lock().flush();
        }
    }
}
