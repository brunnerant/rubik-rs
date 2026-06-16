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
    solve::kociemba::pruning::{PruningTableR, PruningTableSR},
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

    // pruning tables phase 1
    pub prune_eolr: PruningTableR<EOLR>,
    pub prune_co: PruningTableR<CO>,
    pub prune_eolr_co: PruningTableSR<EOLR, CO>,

    // pruning tables phase 2
    pub prune_cp: PruningTableR<CP>,
    pub prune_ep8: PruningTableR<EP8>,
    pub prune_ep4: PruningTableR<EP4>,
    pub prune_cp_ep8: PruningTableSR<CP, EP8>,
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

        let eolr_raw_mv = RawCoordMoveTable::build();
        println!("building eolr pruning table");
        let prune_eolr = PruningTableR::build(&ALL_MOVES, &eolr_raw_mv);
        println!("building co pruning table");
        let prune_co = PruningTableR::build(&ALL_MOVES, &co_mv);
        println!("building eolr-co pruning table");
        let prune_eolr_co =
            PruningTableSR::build(&sym, &ALL_MOVES, &eolr_coord, &eolr_mv, &co_mv, &co_sym);

        let cp_raw_mv = RawCoordMoveTable::build();
        println!("building cp pruning table");
        let prune_cp = PruningTableR::build(&PHASE2_MOVES, &cp_raw_mv);
        println!("building ep8 pruning table");
        let prune_ep8 = PruningTableR::build(&PHASE2_MOVES, &ep8_mv);
        println!("building ep4 pruning table");
        let prune_ep4 = PruningTableR::build(&PHASE2_MOVES, &ep4_mv);
        println!("building cp-ep8 pruning table");
        let prune_cp_ep8 =
            PruningTableSR::build(&sym, &PHASE2_MOVES, &cp_coord, &cp_mv, &ep8_mv, &ep8_sym);

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
            prune_eolr_co,
            prune_eolr,
            prune_co,
            prune_cp,
            prune_ep8,
            prune_ep4,
            prune_cp_ep8,
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
        self.prune_eolr_co
            .to_file(folder.join("prune-eolr-co.bin"))?;
        self.prune_eolr.to_file(folder.join("prune-eolr.bin"))?;
        self.prune_co.to_file(folder.join("prune-co.bin"))?;
        self.prune_cp.to_file(folder.join("prune-cp.bin"))?;
        self.prune_ep8.to_file(folder.join("prune-ep8.bin"))?;
        self.prune_ep4.to_file(folder.join("prune-ep4.bin"))?;
        self.prune_cp_ep8.to_file(folder.join("prune-cp-ep8.bin"))
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
        let prune_eolr_co = BinarySerde::from_file(folder.join("prune-eolr-co.bin"))?;
        let prune_eolr = BinarySerde::from_file(folder.join("prune-eolr.bin"))?;
        let prune_co = BinarySerde::from_file(folder.join("prune-co.bin"))?;
        let prune_cp = BinarySerde::from_file(folder.join("prune-cp.bin"))?;
        let prune_ep8 = BinarySerde::from_file(folder.join("prune-ep8.bin"))?;
        let prune_ep4 = BinarySerde::from_file(folder.join("prune-ep4.bin"))?;
        let prune_cp_ep8 = BinarySerde::from_file(folder.join("prune-cp-ep8.bin"))?;
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
            prune_eolr_co,
            prune_eolr,
            prune_co,
            prune_cp,
            prune_ep8,
            prune_ep4,
            prune_cp_ep8,
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

impl Phase1 {
    pub fn from_state(state: &State, tables: &Tables) -> Self {
        Self {
            eolr: tables
                .eolr_coord
                .raw_to_sym(EOLR::from_state(state).coord()),
            co: CO::from_state(state).coord(),
        }
    }

    pub fn mv(&self, mv: u8, tables: &Tables) -> Self {
        Self {
            eolr: tables.eolr_mv.coord_mv(self.eolr, mv, &tables.sym),
            co: tables.co_mv.coord_mv(self.co, mv),
        }
    }

    pub fn depth(&self, tables: &Tables) -> u8 {
        let mut coords = *self;
        let mut num_moves = 0;
        let mut next_d = (coords.depth_mod_3(tables) + 2) % 3;
        while !coords.reached_goal() {
            coords = (0..18)
                .find_map(|i| {
                    let next_coords = coords.mv(i, tables);
                    (next_coords.depth_mod_3(tables) == next_d).then_some(next_coords)
                })
                .expect("invalid pruning table: no move was found that decreases the distance");
            num_moves += 1;
            next_d = (next_d + 2) % 3;
        }
        num_moves
    }

    pub fn depth_mod_3(&self, tables: &Tables) -> u8 {
        tables
            .prune_eolr_co
            .dist(self.eolr, self.co, &tables.sym, &tables.co_sym)
    }

    pub fn reached_goal(&self) -> bool {
        EOLR::unpack_sym_coord(self.eolr).0 == 0 && self.co == 0
    }
}

#[derive(Clone, Copy, Default)]
pub struct Phase2 {
    pub cp: u16,
    pub ep8: u16,
    pub ep4: u8,
}

impl Phase2 {
    pub fn from_state(state: &State, tables: &Tables) -> Self {
        Self {
            cp: tables.cp_coord.raw_to_sym(CP::from_state(state).coord()),
            ep8: EP8::from_state(state).coord(),
            ep4: EP4::from_state(state).coord(),
        }
    }

    pub fn mv(&self, mv: u8, tables: &Tables) -> Self {
        Self {
            cp: tables.cp_mv.coord_mv(self.cp, mv, &tables.sym),
            ep8: tables.ep8_mv.coord_mv(self.ep8, mv),
            ep4: tables.ep4_mv.coord_mv(self.ep4, mv),
        }
    }

    pub fn depth_cp_ep8(&self, tables: &Tables) -> u8 {
        let mut num_moves = 0;
        let mut cp = self.cp;
        let mut ep8 = self.ep8;
        let mut next_d = (tables
            .prune_cp_ep8
            .dist(cp, ep8, &tables.sym, &tables.ep8_sym)
            + 2)
            % 3;
        while CP::unpack_sym_coord(cp).0 != 0 || ep8 != 0 {
            let (next_cp, next_ep8) = PHASE2_MOVES
                .iter()
                .find_map(|&i| {
                    let next_cp = tables.cp_mv.coord_mv(cp, i, &tables.sym);
                    let next_ep8 = tables.ep8_mv.coord_mv(ep8, i);
                    (tables
                        .prune_cp_ep8
                        .dist(next_cp, next_ep8, &tables.sym, &tables.ep8_sym)
                        == next_d)
                        .then_some((next_cp, next_ep8))
                })
                .expect("invalid pruning table: no move was found that decreases the distance");
            num_moves += 1;
            cp = next_cp;
            ep8 = next_ep8;
            next_d = (next_d + 2) % 3;
        }
        num_moves
    }

    pub fn depth_cp_ep8_mod_3(&self, tables: &Tables) -> u8 {
        tables
            .prune_cp_ep8
            .dist(self.cp, self.ep8, &tables.sym, &tables.ep8_sym)
    }

    pub fn reached_goal(&self) -> bool {
        self.cp == 0 && self.ep8 == 0 && self.ep4 == 0
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
