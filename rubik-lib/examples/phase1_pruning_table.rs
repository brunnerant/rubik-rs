use std::path::Path;

use rubik_lib::{
    core::io::BinarySerde,
    solve::kociemba::{self, phase1},
};

fn main() {
    let coords =
        kociemba::Coords::from_folder(Path::new("data/kociemba")).expect("failed to load coords");
    let pruning_table = phase1::PruningTable::build(&coords);
    print!("Writing the pruning table to 'data/kociemba/phase1-pruning.bin'...");
    pruning_table
        .to_file("data/kociemba/phase1-pruning.bin")
        .expect("failed to write to file");
    println!(" Done.");
}
