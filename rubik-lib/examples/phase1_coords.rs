use std::path::Path;

use rubik_lib::solve::kociemba::phase1;

fn main() {
    std::fs::create_dir_all("data/kociemba/phase1").expect("failed to create data folder");
    phase1::Coords::build()
        .to_folder(Path::new("data/kociemba/phase1"))
        .expect("failed to save the coordinates to 'data/phase1'");
}
