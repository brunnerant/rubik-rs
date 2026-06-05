use std::{
    fs::File,
    io::{BufWriter, Write},
};

use rubik_lib::solve::kociemba::phase1;

fn main() {
    std::fs::create_dir_all("data").expect("failed to create data folder");
    let coords = phase1::Coords::build();
    let pruning_table = phase1::PruningTable::build(&coords);
    let mut writer =
        BufWriter::new(File::create("data/phase1-pruning.bin").expect("couldn't open file"));
    writer
        .write_all(pruning_table.buffer())
        .expect("failed to write to file");
}
