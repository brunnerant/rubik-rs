//! In phase1, the cube is brought to a state where all the corners and edges are correctly oriented,
//! and where the LR slice has a permutation of the correct edges.
//! It does so by using EO, CO, and LR coordinates and bringing them down to zero.

use std::{collections::HashSet, io::Write, marker::PhantomData};

use num::Zero;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    algebra::{
        coord::Coord,
        move_table::{RawCoordMoveTable, RawCoordSymTable, SymCoordMoveTable},
        sym::Symmetries,
        sym_coord::SymCoordTable,
    },
    core::io::BinarySerde,
};

pub struct PruningTable<C1: Coord, C2: Coord> {
    /// This is a generic pruning table implementation that takes two coordinates as input
    /// and outputs the depth from the initial state, modulo 3. The modulo 3 is used
    /// to compress the table thanks to the face adjacent states in the state graph can only
    /// differ by one in terms of depth.
    /// The table is compressed using symmetries on the first coordinate. Let the input
    /// state belong to the first coordinate class (i, j), where i is the index of the
    /// representant, and j the symmetry. Let k be the second coordinate. If we apply
    /// symmetry inv(j) on the state, it gives us the class (i, 0), and k'. i and k'
    /// are used to index the table, which compresses the full table by about 16.
    table: Vec<u8>,
    _c1: PhantomData<C1>,
    _c2: PhantomData<C2>,
}

impl<C1: Coord, C2: Coord> PruningTable<C1, C2> {
    pub fn build(
        sym: &Symmetries,
        mvs: &[u8],
        c1_coord: &SymCoordTable<C1>,
        c1_mv: &SymCoordMoveTable<C1>,
        c2_mv: &RawCoordMoveTable<C2>,
        c2_sym: &RawCoordSymTable<C2>,
    ) -> Self {
        PruningTableBuilder::new(sym, mvs, c1_coord, c1_mv, c2_mv, c2_sym).build()
    }

    pub fn dist(
        &self,
        c1: C1::Raw,
        c2: C2::Raw,
        sym: &Symmetries,
        c2_sym: &RawCoordSymTable<C2>,
    ) -> u8 {
        let (i, j) = C1::unpack_sym_coord(c1);
        let k = c2_sym.coord_sym(c2, sym.inv(j));
        let idx = C1::sym_to_usize(i) * C2::raw_to_usize(C2::RAW_SIZE) + C2::raw_to_usize(k);
        let byte_idx = idx >> 2;
        let bit_idx = (idx & 0b11) << 1;
        (self.table[byte_idx] >> bit_idx) & 0b11
    }
}

impl<C1: Coord, C2: Coord> BinarySerde for PruningTable<C1, C2> {
    fn from_binary(buffer: &[u8]) -> Option<Self> {
        Some(Self {
            table: buffer.to_vec(),
            _c1: Default::default(),
            _c2: Default::default(),
        })
    }

    fn to_binary(&self) -> Vec<u8> {
        self.table.clone()
    }
}

struct PruningTableBuilder<'a, C1: Coord, C2: Coord> {
    table: Vec<u8>,
    total_entries: usize,
    num_entries: usize,
    sym: &'a Symmetries,
    mvs: &'a [u8],
    c1_coord: &'a SymCoordTable<C1>,
    c1_mv: &'a SymCoordMoveTable<C1>,
    c2_mv: &'a RawCoordMoveTable<C2>,
    c2_sym: &'a RawCoordSymTable<C2>,
}

impl<'a, C1: Coord, C2: Coord> PruningTableBuilder<'a, C1, C2> {
    fn new(
        sym: &'a Symmetries,
        mvs: &'a [u8],
        c1_coord: &'a SymCoordTable<C1>,
        c1_mv: &'a SymCoordMoveTable<C1>,
        c2_mv: &'a RawCoordMoveTable<C2>,
        c2_sym: &'a RawCoordSymTable<C2>,
    ) -> Self {
        let total_entries = C1::sym_to_usize(C1::SYM_SIZE) * C2::raw_to_usize(C2::RAW_SIZE);
        let table = vec![!0; total_entries.div_ceil(4)];
        PruningTableBuilder {
            table,
            total_entries,
            num_entries: 0,
            sym,
            mvs,
            c1_coord,
            c1_mv,
            c2_mv,
            c2_sym,
        }
    }

    fn build(mut self) -> PruningTable<C1, C2> {
        let mut new_entries: HashSet<_> = self.insert(Zero::zero(), Zero::zero()).collect(); // add the initial state
        let mut depth = 0;
        loop {
            if self.fill_new_entries(&new_entries, depth) {
                break;
            }
            depth += 1;

            let builder = &self;
            new_entries = new_entries
                .into_par_iter()
                .flat_map_iter(|(c1, c2)| {
                    let c1 = C1::pack_sym_coord(c1, 0);
                    builder.mvs.iter().flat_map(move |&mv| {
                        let next_c1 = builder.c1_mv.coord_mv(c1, mv, builder.sym);
                        let next_c2 = builder.c2_mv.coord_mv(c2, mv);
                        builder.insert(next_c1, next_c2)
                    })
                })
                .collect();
        }
        PruningTable {
            table: self.table,
            _c1: Default::default(),
            _c2: Default::default(),
        }
    }

    fn get(&self, idx: usize) -> u8 {
        let byte_idx = idx >> 2;
        let bit_idx = (idx & 0b11) << 1;
        (self.table[byte_idx] >> bit_idx) & 0b11
    }

    fn set(&mut self, idx: usize, val: u8) {
        let byte_idx = idx >> 2;
        let bit_idx = (idx & 0b11) << 1;
        self.table[byte_idx] &= !(0b11 << bit_idx);
        self.table[byte_idx] |= (val & 0b11) << bit_idx;
    }

    fn insert(&self, c1: C1::Raw, c2: C2::Raw) -> impl Iterator<Item = (C1::Sym, C2::Raw)> {
        let (i, j) = C1::unpack_sym_coord(c1);
        self.c1_coord.internal_sym(i).iter().filter_map(move |&k| {
            let s = self.sym.prod(self.sym.inv(j), k);
            let k = self.c2_sym.coord_sym(c2, s);
            let idx = C1::sym_to_usize(i) * C2::raw_to_usize(C2::RAW_SIZE) + C2::raw_to_usize(k);
            (self.get(idx) == 0b11).then_some((i, k))
        })
    }

    fn fill_new_entries(&mut self, new_entries: &HashSet<(C1::Sym, C2::Raw)>, d: u8) -> bool {
        println!("\rdepth {d} done. {} new entries.", new_entries.len());
        let _ = std::io::stdout().lock().flush();
        let d = d % 3;
        for &(c1, c2) in new_entries {
            let idx = C1::sym_to_usize(c1) * C2::raw_to_usize(C2::RAW_SIZE) + C2::raw_to_usize(c2);
            self.set(idx, d);
            self.num_entries += 1;
        }
        self.num_entries == self.total_entries
    }
}
