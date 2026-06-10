use rubik_lib::solve::kociemba::{self};

fn main() {
    std::fs::create_dir_all("data/kociemba").expect("failed to create data folder");
    kociemba::Tables::build()
        .to_folder("data/kociemba")
        .expect("failed to save the coordinates to 'data/kociemba'");
}
