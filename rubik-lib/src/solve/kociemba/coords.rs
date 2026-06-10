use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use crate::{
    algebra::{
        coord::{CO, CP, Coord, EOLR, EP4, EP8},
        move_table::{RawCoordMoveTable, RawCoordSymTable, SymCoordMoveTable},
        sym::Symmetries,
        sym_coord::SymCoordTable,
    },
    core::{io::BinarySerde, state::State},
    solve::kociemba::pruning::PruningTable,
};

pub const ALL_MOVES: [u8; 18] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
pub const PHASE2_MOVES: [u8; 10] = [0, 1, 2, 3, 4, 5, 8, 11, 14, 17];

pub fn is_phase2_move(mv: u8) -> bool {
    !(mv / 3 >= 2 && mv % 3 < 2)
}

pub struct Tables {
    // the 16 symmetries that reduce the state space
    pub sym: Symmetries,

    // phase 1 coordinates
    pub eolr_coord: SymCoordTable<EOLR>,
    pub eolr_mv: SymCoordMoveTable<EOLR>,
    pub co_mv: RawCoordMoveTable<CO>,
    pub co_sym: RawCoordSymTable<CO>,

    // phase 2 coordinates
    pub cp_coord: SymCoordTable<CP>,
    pub cp_mv: SymCoordMoveTable<CP>,
    pub ep8_mv: RawCoordMoveTable<EP8>,
    pub ep8_sym: RawCoordSymTable<EP8>,
    pub ep4_mv: RawCoordMoveTable<EP4>,

    // pruning tables
    pub phase1_pruning: PruningTable<EOLR, CO>,
    pub phase2_pruning: PruningTable<CP, EP8>,
}

pub trait Coords {
    type Tables;
    fn from_state(state: &State, tables: &Self::Tables) -> Self;
    fn mv(&self, mv: u8, tables: &Self::Tables) -> Self;
    fn min_dist(&self, tables: &Self::Tables) -> u8;
    fn reached_goal(&self) -> bool;
}

impl Tables {
    pub fn build() -> Tables {
        let sym = Symmetries::sub16();
        let eolr_coord = SymCoordTable::build(&sym);
        let eolr_mv = SymCoordMoveTable::build(&eolr_coord);
        let co_mv = RawCoordMoveTable::build();
        let co_sym = RawCoordSymTable::build(&sym);
        let cp_coord = SymCoordTable::build(&sym);
        let cp_mv = SymCoordMoveTable::build(&cp_coord);
        let ep8_mv = RawCoordMoveTable::build();
        let ep8_sym = RawCoordSymTable::build(&sym);
        let ep4_mv = RawCoordMoveTable::build();
        let phase1_pruning =
            PruningTable::build(&sym, &ALL_MOVES, &eolr_coord, &eolr_mv, &co_mv, &co_sym);
        let phase2_pruning =
            PruningTable::build(&sym, &PHASE2_MOVES, &cp_coord, &cp_mv, &ep8_mv, &ep8_sym);
        Self {
            sym,
            eolr_coord,
            eolr_mv,
            co_mv,
            co_sym,
            cp_coord,
            cp_mv,
            ep8_mv,
            ep8_sym,
            ep4_mv,
            phase1_pruning,
            phase2_pruning,
        }
    }

    pub fn to_folder(&self, folder: impl AsRef<Path>) -> std::io::Result<()> {
        let folder = folder.as_ref();
        self.eolr_coord.to_file(folder.join("eolr-sym-coord.bin"))?;
        self.eolr_mv.to_file(folder.join("eolr-sym-coord-mv.bin"))?;
        self.co_mv.to_file(folder.join("co-raw-coord-mv.bin"))?;
        self.co_sym.to_file(folder.join("co-raw-coord-sym.bin"))?;
        self.cp_coord.to_file(folder.join("cp-sym-coord.bin"))?;
        self.cp_mv.to_file(folder.join("cp-sym-coord-mv.bin"))?;
        self.ep8_mv.to_file(folder.join("ep8-raw-coord-mv.bin"))?;
        self.ep8_sym.to_file(folder.join("ep8-raw-coord-sym.bin"))?;
        self.ep4_mv.to_file(folder.join("ep4-raw-coord-mv.bin"))?;
        self.phase1_pruning
            .to_file(folder.join("phase1-pruning.bin"))?;
        self.phase2_pruning
            .to_file(folder.join("phase2-pruning.bin"))
    }

    pub fn from_folder(folder: impl AsRef<Path>) -> std::io::Result<Self> {
        let folder = folder.as_ref();
        let eolr_coord = BinarySerde::from_file(folder.join("eolr-sym-coord.bin"))?;
        let eolr_mv = BinarySerde::from_file(folder.join("eolr-sym-coord-mv.bin"))?;
        let co_mv = BinarySerde::from_file(folder.join("co-raw-coord-mv.bin"))?;
        let co_sym = BinarySerde::from_file(folder.join("co-raw-coord-sym.bin"))?;
        let cp_coord = BinarySerde::from_file(folder.join("cp-sym-coord.bin"))?;
        let cp_mv = BinarySerde::from_file(folder.join("cp-sym-coord-mv.bin"))?;
        let ep8_mv = BinarySerde::from_file(folder.join("ep8-raw-coord-mv.bin"))?;
        let ep8_sym = BinarySerde::from_file(folder.join("ep8-raw-coord-sym.bin"))?;
        let ep4_mv = BinarySerde::from_file(folder.join("ep4-raw-coord-mv.bin"))?;
        let phase1_pruning = BinarySerde::from_file(folder.join("phase1-pruning.bin"))?;
        let phase2_pruning = BinarySerde::from_file(folder.join("phase2-pruning.bin"))?;
        let sym = Symmetries::sub16();

        Ok(Self {
            sym,
            eolr_coord,
            eolr_mv,
            co_mv,
            co_sym,
            cp_coord,
            cp_mv,
            ep8_mv,
            ep8_sym,
            ep4_mv,
            phase1_pruning,
            phase2_pruning,
        })
    }

    pub fn buffer_from_file(file: &Path) -> std::io::Result<Vec<u8>> {
        let mut reader = BufReader::new(File::open(file)?);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}

#[derive(Clone, Copy, Default)]
pub struct Phase1 {
    pub eolr: u32,
    pub co: u16,
}

impl Coords for Phase1 {
    type Tables = Tables;
    fn from_state(state: &State, tables: &Self::Tables) -> Self {
        Self {
            eolr: tables
                .eolr_coord
                .raw_to_sym(EOLR::from_state(state).coord()),
            co: CO::from_state(state).coord(),
        }
    }
    fn mv(&self, mv: u8, tables: &Self::Tables) -> Self {
        Self {
            eolr: tables.eolr_mv.coord_mv(self.eolr, mv, &tables.sym),
            co: tables.co_mv.coord_mv(self.co, mv),
        }
    }
    fn min_dist(&self, tables: &Self::Tables) -> u8 {
        tables
            .phase1_pruning
            .dist(self.eolr, self.co, &tables.sym, &tables.co_sym)
    }
    fn reached_goal(&self) -> bool {
        EOLR::unpack_sym_coord(self.eolr).0 == 0 && self.co == 0
    }
}

#[derive(Clone, Copy, Default)]
pub struct Phase2 {
    pub cp: u16,
    pub ep8: u16,
    pub ep4: u8,
}

impl Coords for Phase2 {
    type Tables = Tables;
    fn from_state(state: &State, tables: &Self::Tables) -> Self {
        Self {
            cp: tables.cp_coord.raw_to_sym(CP::from_state(state).coord()),
            ep8: EP8::from_state(state).coord(),
            ep4: EP4::from_state(state).coord(),
        }
    }
    fn mv(&self, mv: u8, tables: &Self::Tables) -> Self {
        Self {
            cp: tables.cp_mv.coord_mv(self.cp, mv, &tables.sym),
            ep8: tables.ep8_mv.coord_mv(self.ep8, mv),
            ep4: tables.ep4_mv.coord_mv(self.ep4, mv),
        }
    }
    fn min_dist(&self, tables: &Self::Tables) -> u8 {
        tables
            .phase2_pruning
            .dist(self.cp, self.ep8, &tables.sym, &tables.ep8_sym)
    }
    fn reached_goal(&self) -> bool {
        CP::unpack_sym_coord(self.cp).0 == 0 && self.ep8 == 0 && self.ep4 == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::solve::kociemba::coords::{PHASE2_MOVES, is_phase2_move};

    #[test]
    fn phase2_moves() {
        let moves = (0..18).filter(|&i| is_phase2_move(i)).collect::<Vec<_>>();
        assert_eq!(&PHASE2_MOVES, &moves[..])
    }
}
