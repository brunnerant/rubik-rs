use std::{
    fs::File,
    io::{BufReader, Read},
};

use itertools::join;
use rubik_lib::{
    algebra::coord::{CO, Coord, EOLR},
    core::{moves::Move, scramble::scramble, state::State},
    solve::kociemba::phase1::{self},
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
        let mv = (0..18).find(|&i| {
            let next_eolr = coords.eolr_mv.coord_mv(eolr, i, &coords.sym);
            let next_co = coords.co_mv.coord_mv(co, i);
            pruning_table.dist(next_eolr, next_co, &coords) == next_d
        });
        match mv {
            Some(i) => {
                moves.push(Move::BASIC_MOVES[i as usize]);
                eolr = coords.eolr_mv.coord_mv(eolr, i, &coords.sym);
                co = coords.co_mv.coord_mv(co, i);
                next_d = (pruning_table.dist(eolr, co, &coords) + 2) % 3;
            }
            None => panic!(),
        }
    }
    moves
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
        println!("{}", join(moves, " "));
    }
}
