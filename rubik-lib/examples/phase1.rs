use std::{
    fs::File,
    io::{BufReader, Read},
};

use itertools::join;
use rubik_lib::{
    algebra::coord::{CO, Coord, EOLR},
    core::{moves::Move, scramble::scramble, state::State},
    solve::kociemba::phase1,
};

fn solve_phase1(
    state: State,
    coords: &phase1::Coords,
    pruning_table: &phase1::PruningTable,
) -> Vec<Move> {
    let mut moves = Vec::new();
    let mut eolr = coords.eolr_coord.sym_coord(state, &coords.sym);
    let mut co = CO::from_state(&state).repr();
    let mut next_d = (pruning_table.dist(eolr, co, &coords) + 2) % 3;
    while EOLR::unpack_sym_coord(eolr).0 != 0 || co != 0 {
        let (i, next_eolr, next_co) = (0..18)
            .find_map(|i| {
                let next_eolr = coords.eolr_mv.coord_mv(eolr, i, &coords.sym);
                let next_co = coords.co_mv.coord_mv(co, i);
                (pruning_table.dist(next_eolr, next_co, &coords) == next_d)
                    .then_some((i, next_eolr, next_co))
            })
            .expect("invalid pruning table: no move was found that decreases the distance");
        moves.push(Move::BASIC_MOVES[i as usize]);
        eolr = next_eolr;
        co = next_co;
        next_d = (next_d + 2) % 3;
    }
    moves
}

fn check_phase1(mut state: State, moves: &Vec<Move>, coords: &phase1::Coords) {
    for &mv in moves {
        state = state * mv;
    }
    let (_, j) = EOLR::unpack_sym_coord(coords.eolr_coord.sym_coord(state, &coords.sym));
    state = coords.sym.conj_inv(state, j);
    assert_eq!(0, EOLR::from_state(&state).repr());
    assert_eq!(0, CO::from_state(&state).repr());
}

fn main() {
    let mut buf_reader =
        BufReader::new(File::open("data/phase1-pruning.bin").expect("unable to open file"));
    let mut buffer = Vec::new();
    buf_reader
        .read_to_end(&mut buffer)
        .expect("unable to read file");
    let pruning_table = phase1::PruningTable::from_buffer(buffer);
    let coords = phase1::Coords::build();

    for _ in 0..100 {
        let (_, state) = scramble(100);
        let moves = solve_phase1(state, &coords, &pruning_table);
        check_phase1(state, &moves, &coords);
        println!("{}", join(moves, " "));
    }
}
