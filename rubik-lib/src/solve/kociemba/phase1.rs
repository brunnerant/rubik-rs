//! In phase1, the cube is brought to a state where all the corners and edges are correctly oriented,
//! and where the LR slice has a permutation of the correct edges.
//! It does so by using EO, CO, and LR coordinates and bringing them down to zero.

use std::{
    collections::HashSet,
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::algebra::{
    coord::{CO, Coord, EOLR},
    move_table::{RawCoordMoveTable, RawCoordSymTable, SymCoordMoveTable},
    sym::Symmetries,
    sym_coord::SymCoordTable,
};

pub type EOLRRaw = <EOLR as Coord>::Raw;
pub type EOLRSym = <EOLR as Coord>::Sym;
pub type CORaw = <CO as Coord>::Raw;

pub struct Coords {
    pub eolr_coord: SymCoordTable<EOLR>,
    pub eolr_mv: SymCoordMoveTable<EOLR>,
    pub co_mv: RawCoordMoveTable<CO>,
    pub co_sym: RawCoordSymTable<CO>,
    pub sym: Symmetries,
}

impl Coords {
    pub fn build() -> Coords {
        let sym = Symmetries::sub16();
        let eolr_coord = SymCoordTable::build(&sym);
        Self {
            eolr_mv: SymCoordMoveTable::build(&eolr_coord, &sym),
            eolr_coord,
            co_mv: RawCoordMoveTable::build(),
            co_sym: RawCoordSymTable::build(&sym),
            sym,
        }
    }

    pub fn to_folder(&self, folder: &Path) -> std::io::Result<()> {
        Self::buffer_to_file(
            &folder.join("eolr-sym-coord.bin"),
            &self.eolr_coord.serialize(),
        )?;
        Self::buffer_to_file(
            &folder.join("eolr-sym-coord-mv.bin"),
            &self.eolr_mv.serialize(),
        )?;
        Self::buffer_to_file(&folder.join("co-raw-coord-mv.bin"), &self.co_mv.serialize())?;
        Self::buffer_to_file(
            &folder.join("co-raw-coord-sym.bin"),
            &self.co_sym.serialize(),
        )
    }

    fn buffer_to_file(file: &Path, buffer: &[u8]) -> std::io::Result<()> {
        let mut writer = BufWriter::new(File::create(file)?);
        writer.write_all(buffer)
    }

    pub fn from_folder(folder: &Path) -> std::io::Result<Self> {
        let eolr_coord = SymCoordTable::deserialize(&Self::buffer_from_file(
            &folder.join("eolr-sym-coord.bin"),
        )?)
        .ok_or(std::io::ErrorKind::InvalidData)?;
        let eolr_mv = SymCoordMoveTable::deserialize(&Self::buffer_from_file(
            &folder.join("eolr-sym-coord-mv.bin"),
        )?)
        .ok_or(std::io::ErrorKind::InvalidData)?;
        let co_mv = RawCoordMoveTable::deserialize(&Self::buffer_from_file(
            &folder.join("co-raw-coord-mv.bin"),
        )?)
        .ok_or(std::io::ErrorKind::InvalidData)?;
        let co_sym = RawCoordSymTable::deserialize(&Self::buffer_from_file(
            &folder.join("co-raw-coord-sym.bin"),
        )?)
        .ok_or(std::io::ErrorKind::InvalidData)?;
        let sym = Symmetries::sub16();

        Ok(Self {
            eolr_coord,
            eolr_mv,
            co_mv,
            co_sym,
            sym,
        })
    }

    pub fn buffer_from_file(file: &Path) -> std::io::Result<Vec<u8>> {
        let mut reader = BufReader::new(File::open(file)?);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}

pub struct PruningTable {
    /// The table is indexed using the EOLR + CO coordinates. If (y, i) is the EOLR sym
    /// coord for a state, then find the equivalent (y, 0) coordinate by symmetry, and
    /// the corresponding CO coordinate x. Index using y * 2187 + x.
    /// Each entry contains two bits that represent the depth modulo 3.
    /// Modulo 3 is enough because the move count can change by at most 1 after a move.
    table: Vec<u8>,
}

impl PruningTable {
    pub fn build(coords: &Coords) -> PruningTable {
        PruningTableBuilder::new(coords).build()
    }

    pub fn dist(&self, eolr: EOLRRaw, co: CORaw, coords: &Coords) -> u8 {
        let (i, j) = EOLR::unpack_sym_coord(eolr);
        let co = coords.co_sym.coord_sym(co, coords.sym.inv(j));
        let idx = i as u32 * CO::RAW_SIZE as u32 + co as u32;
        let byte_idx = (idx >> 2) as usize;
        let bit_idx = (idx & 0b11) << 1;
        (self.table[byte_idx] >> bit_idx) & 0b11
    }

    pub fn deserialize(buffer: &[u8]) -> Option<Self> {
        Some(Self {
            table: buffer.to_vec(),
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.table.clone()
    }
}

struct PruningTableBuilder<'a> {
    table: Vec<u8>,
    total_entries: usize,
    num_entries: usize,
    coords: &'a Coords,
}

impl<'a> PruningTableBuilder<'a> {
    fn new(coords: &'a Coords) -> PruningTableBuilder<'a> {
        let total_entries = EOLR::sym_to_usize(EOLR::SYM_SIZE) * CO::raw_to_usize(CO::RAW_SIZE);
        let table = vec![!0; total_entries.div_ceil(4)];
        PruningTableBuilder {
            table,
            total_entries,
            num_entries: 0,
            coords,
        }
    }

    fn build(mut self) -> PruningTable {
        let mut new_entries: HashSet<_> = self.insert(0, 0).collect(); // add the initial state
        let mut depth = 0;
        loop {
            if self.fill_new_entries(&new_entries, depth) {
                break;
            }
            depth += 1;

            let builder = &self;
            new_entries = new_entries
                .into_par_iter()
                .flat_map_iter(|(eolr, co)| {
                    let eolr = EOLR::pack_sym_coord(eolr, 0);
                    (0..18).flat_map(move |mv| {
                        let next_eolr =
                            builder
                                .coords
                                .eolr_mv
                                .coord_mv(eolr, mv, &builder.coords.sym);
                        let next_co = builder.coords.co_mv.coord_mv(co, mv);
                        builder.insert(next_eolr, next_co)
                    })
                })
                .collect();
        }
        PruningTable { table: self.table }
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

    fn insert(&self, eolr: EOLRRaw, co: CORaw) -> impl Iterator<Item = (EOLRSym, CORaw)> {
        let (i, j) = EOLR::unpack_sym_coord(eolr);
        self.coords
            .eolr_coord
            .internal_sym(i)
            .iter()
            .filter_map(move |&k| {
                let s = self.coords.sym.prod(self.coords.sym.inv(j), k);
                let next_co = self.coords.co_sym.coord_sym(co, s);
                let idx = i as u32 * CO::RAW_SIZE as u32 + next_co as u32;
                (self.get(idx) == 0b11).then_some((i, next_co))
            })
    }

    fn fill_new_entries(&mut self, new_entries: &HashSet<(EOLRSym, CORaw)>, d: u8) -> bool {
        println!("\rdepth {d} done. {} new entries.", new_entries.len());
        let _ = std::io::stdout().lock().flush();
        let d = d % 3;
        for &(eolr, co) in new_entries {
            let idx = eolr as u32 * CO::RAW_SIZE as u32 + co as u32;
            self.set(idx, d);
            self.num_entries += 1;
        }
        self.num_entries == self.total_entries
    }
}
