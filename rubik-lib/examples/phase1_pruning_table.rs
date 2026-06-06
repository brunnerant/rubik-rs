use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use rubik_lib::solve::kociemba::phase1;

fn main() {
    let coords = phase1::Coords::from_folder(Path::new("data/kociemba/phase1"))
        .expect("failed to load coords");
    let pruning_table = phase1::PruningTable::build(&coords);
    let mut writer = BufWriter::new(
        File::create("data/kociemba/phase1/pruning.bin").expect("couldn't open file"),
    );
    print!("Writing the pruning table to 'data/kociemba/phase1/pruning.bin'...");
    writer
        .write_all(&pruning_table.serialize())
        .expect("failed to write to file");
    println!(" Done.");
}
