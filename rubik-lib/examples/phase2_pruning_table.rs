use std::path::Path;

use rubik_lib::{
    algebra::coord::{CP, EP8},
    core::io::BinarySerde,
    solve::kociemba::{self, pruning::PruningTable},
};

fn main() {
    let moves = [0, 1, 2, 3, 4, 5, 8, 11, 14, 17];
    let coords =
        kociemba::Coords::from_folder(Path::new("data/kociemba")).expect("failed to load coords");
    let pruning_table = PruningTable::<CP, EP8>::build(
        &coords.sym,
        &moves,
        &coords.cp_coord,
        &coords.cp_mv,
        &coords.ep8_mv,
        &coords.ep8_sym,
    );
    print!("Writing the pruning table to 'data/kociemba/phase2-pruning.bin'...");
    pruning_table
        .to_file("data/kociemba/phase2-pruning.bin")
        .expect("failed to write to file");
    println!(" Done.");
}
