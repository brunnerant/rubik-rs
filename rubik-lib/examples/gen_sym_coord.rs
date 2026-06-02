use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use rubik_lib::{
    model::{
        bits::BitField,
        coord::{CO, Coord, EO, EOLR},
        sym::Symmetries,
    },
    solve::kociemba::phase1::SymCoordTable,
};

fn create_table<C: Coord, SymC: BitField>(sym: &Symmetries, output_file: &str) {
    let output_file = Path::new(output_file);
    print!("Creating table '{:?}'.", output_file);
    let _ = std::io::stdout().lock().flush();
    let sym_table = SymCoordTable::<C, SymC>::build(sym);
    println!(" Done. ({} elements)", sym_table.size());
    let mut writer = BufWriter::new(File::create(output_file).expect("couldn't open file"));
    sym_table
        .serialize(&mut writer)
        .expect("error while serializing table");
}

fn main() {
    let _ = std::fs::create_dir_all("data");
    create_table::<CO, u8>(&Symmetries::sub16(), "data/co-sym-coord.bin");
    create_table::<EO, u8>(&Symmetries::sub16(), "data/eo-sym-coord.bin");
    create_table::<EOLR, u16>(&Symmetries::sub16(), "data/eolr-sym-coord.bin");
}
