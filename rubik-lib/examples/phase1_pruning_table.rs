use std::path::Path;

use rubik_lib::{
    algebra::coord::{CO, EOLR},
    core::io::BinarySerde,
    solve::kociemba::{self, pruning::PruningTable},
};

fn main() {
    let coords =
        kociemba::Coords::from_folder(Path::new("data/kociemba")).expect("failed to load coords");
    let pruning_table = PruningTable::<EOLR, CO>::build(
        &coords.sym,
        &coords.eolr_coord,
        &coords.eolr_mv,
        &coords.co_mv,
        &coords.co_sym,
    );
    print!("Writing the pruning table to 'data/kociemba/phase1-pruning.bin'...");
    pruning_table
        .to_file("data/kociemba/phase1-pruning.bin")
        .expect("failed to write to file");
    println!(" Done.");
}
