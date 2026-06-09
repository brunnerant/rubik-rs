pub mod coords;
pub mod pruning;

use std::path::Path;

pub use coords::Coords;
use smallvec::{SmallVec, smallvec};

use crate::{
    algebra::coord::{CO, CP, Coord, EOLR, EP8},
    core::io::BinarySerde,
    solve::kociemba::pruning::PruningTable,
};

struct Phase1Record {
    mv: u8,
    last_eolr: u32,
    last_co: u16,
}

struct Phase2Record {
    mv: u8,
}

pub struct Solver {
    coords: Coords,
    phase1_pruning: PruningTable<EOLR, CO>,
    phase2_pruning: PruningTable<CP, EP8>,

    phase1: SmallVec<[Phase1Record; 12]>,
    phase2: SmallVec<[Phase2Record; 18]>,
    best: SmallVec<[u8; 30]>,

    eolr: u32,
    co: u16,
}

impl Solver {
    pub fn from_folder(folder: impl AsRef<Path>) -> std::io::Result<Self> {
        let folder = folder.as_ref();
        let coords = Coords::from_folder(folder)?;
        let phase1_pruning = PruningTable::from_file(folder.join("phase1-pruning.bin"))?;
        let phase2_pruning = PruningTable::from_file(folder.join("phase1-pruning.bin"))?;
        Ok(Self {
            coords,
            phase1_pruning,
            phase2_pruning,
            phase1: smallvec![],
            phase2: smallvec![],
            best: smallvec![],
            eolr: 0,
            co: 0,
        })
    }

    fn search(&mut self, phase1_len: usize) {}

    fn search_phase2(&mut self, phase2_max_len: usize) {}
}

fn phase1_min_len(
    mut eolr: u32,
    mut co: u16,
    coords: &Coords,
    phase1_pruning: &PruningTable<EOLR, CO>,
) -> usize {
    let mut num_moves = 0;
    let mut next_d = (phase1_pruning.dist(eolr, co, &coords.sym, &coords.co_sym) + 2) % 3;
    while EOLR::unpack_sym_coord(eolr).0 != 0 || co != 0 {
        let (i, next_eolr, next_co) = (0..18)
            .find_map(|i| {
                let next_eolr = coords.eolr_mv.coord_mv(eolr, i, &coords.sym);
                let next_co = coords.co_mv.coord_mv(co, i);
                (phase1_pruning.dist(next_eolr, next_co, &coords.sym, &coords.co_sym) == next_d)
                    .then_some((i, next_eolr, next_co))
            })
            .expect("invalid pruning table: no move was found that decreases the distance");
        num_moves += 1;
        eolr = next_eolr;
        co = next_co;
        next_d = (next_d + 2) % 3;
    }
    num_moves
}
