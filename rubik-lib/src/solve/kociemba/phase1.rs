//! In phase1, the cube is brought to a state where all the corners and edges are correctly oriented,
//! and where the LR slice has a permutation of the correct edges.
//! It does so by using EO, CO, and LR coordinates and bringing them down to zero.

use std::collections::HashSet;

use crate::algebra::{
        coord::{CO, Coord, EOLR},
        move_table::{RawCoordMoveTable, RawCoordSymTable, SymCoordMoveTable},
        sym::Symmetries, sym_coord::SymCoordTable,
    };

pub type EOLRRaw = <EOLR as Coord>::Raw;
pub type EOLRSym = <EOLR as Coord>::Sym;
pub type CORaw = <CO as Coord>::Raw;

pub struct PruningTable {
    /// The table is indexed using the EOLR + CO coordinates. If (y, i) is the EOLR sym
    /// coord for a state, then find the equivalent (y, 0) coordinate by symmetry.
    table: Vec<u8>,
    sym_table: RawCoordSymTable<CO>,
}

impl PruningTable {
    pub fn build(
        eolr_coord: &SymCoordTable<EOLR>,
        eolr_mv: &SymCoordMoveTable<EOLR>,
        co_mv: &RawCoordMoveTable<CO>,
        sym: &Symmetries,
    ) -> PruningTable {
        let total_entries = EOLR::sym_to_usize(EOLR::SYM_SIZE) * CO::raw_to_usize(CO::RAW_SIZE);
        let mut table = Self {
            table: vec![!0; total_entries / 4],
            sym_table: RawCoordSymTable::build(sym),
        };
        let mut curr_depth = HashSet::new();
        let mut next_depth = HashSet::new();
        let mut num_entries = 0;
        table.insert(0, 0, 0, &mut num_entries, total_entries, &mut curr_depth, eolr_coord, sym);
        for d in 0..12 {
            for (eolr, co) in curr_depth.drain() {
                let eolr = EOLR::pack_sym_coord(eolr, 0);
                for mv in 0..18 {
                    let next_eolr = eolr_mv.coord_mv(eolr, mv, sym);
                    let next_co = co_mv.coord_mv(co, mv);
                    table.insert(next_eolr, next_co, d + 1, &mut num_entries, total_entries, &mut next_depth, eolr_coord, sym);
                }
            }
            std::mem::swap(&mut curr_depth, &mut next_depth);
        }
        assert_eq!(num_entries, total_entries);
        table
    }

    pub fn dist(&self, eolr: EOLRRaw, co: CORaw) -> u8 {
        self.get(self.idx(eolr, co))
    }

    fn idx(&self, eolr: EOLRRaw, co: CORaw) -> u32 {
        let (i, j) = EOLR::unpack_sym_coord(eolr);
        let co = self.sym_table.coord_sym_inv(co, j);
        i as u32 * CO::RAW_SIZE as u32 + co as u32
    }

    fn get(&self, idx: u32) -> u8 {
        let byte_idx = (idx >> 2) as usize;
        let bit_idx = (idx & 0b11) << 1;
        (self.table[byte_idx] >> bit_idx) & 0b11
    }

    fn set(&mut self, idx: u32, val: u8) {
        let byte_idx = (idx >> 2) as usize;
        let bit_idx = (idx & 0b11) << 1;
        self.table[byte_idx] &= !(0b11 << bit_idx);
        self.table[byte_idx] |= (val & 0b11) << bit_idx;
    }

    fn insert(
        &mut self,
        eolr: EOLRRaw,
        co: CORaw,
        d: u8,
        num_entries: &mut usize,
        total_entries: usize,
        to_visit: &mut HashSet<(EOLRSym, CORaw)>,
        eolr_coord: &SymCoordTable<EOLR>,
        sym: &Symmetries,
    ) {
        let (i, j) = EOLR::unpack_sym_coord(eolr);
        for k in eolr_coord.internal_sym(i, sym) {
            let s = sym.prod(j, sym.inv(k));
            let next_co = self.sym_table.coord_sym_inv(co, s);
            let idx = i as u32 * CO::RAW_SIZE as u32 + next_co as u32;
            if self.get(idx) == 0b11 {
                self.set(idx, d % 3);
                to_visit.insert((i, next_co));
                *num_entries += 1;
                if *num_entries % 1000 == 0 {
                    let num_entries = *num_entries / 1_000_000;
                    let total_entries = total_entries / 1_000_000;
                    print!("\rdepth {}: {num_entries}M / {total_entries}M", d);
                }
            }
        }
    }
}
