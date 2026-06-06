//! In phase1, the cube is brought to a state where all the corners and edges are correctly oriented,
//! and where the LR slice has a permutation of the correct edges.
//! It does so by using EO, CO, and LR coordinates and bringing them down to zero.

use std::{
    collections::HashSet,
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

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
    new_at_depth: usize,
    to_visit: HashSet<(EOLRSym, CORaw)>,
    depth: u8,
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
            new_at_depth: 0,
            to_visit: HashSet::new(),
            depth: 0,
            coords,
        }
    }

    fn build(mut self) -> PruningTable {
        self.insert(0, 0); // add the initial state
        loop {
            println!(
                "\rdepth {} done. {} new entries.",
                self.depth, self.new_at_depth
            );
            self.new_at_depth = 0;
            self.depth += 1;
            if self.num_entries == self.total_entries {
                break;
            }

            let mut visit_now = HashSet::new();
            std::mem::swap(&mut visit_now, &mut self.to_visit);
            for (eolr, co) in visit_now.drain() {
                let eolr = EOLR::pack_sym_coord(eolr, 0);
                for mv in 0..18 {
                    let next_eolr = self.coords.eolr_mv.coord_mv(eolr, mv, &self.coords.sym);
                    let next_co = self.coords.co_mv.coord_mv(co, mv);
                    self.insert(next_eolr, next_co);
                }
            }
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

    fn insert(&mut self, eolr: EOLRRaw, co: CORaw) {
        let (i, j) = EOLR::unpack_sym_coord(eolr);
        for &k in self.coords.eolr_coord.internal_sym(i) {
            let s = self.coords.sym.prod(self.coords.sym.inv(j), k);
            let next_co = self.coords.co_sym.coord_sym(co, s);
            let idx = i as u32 * CO::RAW_SIZE as u32 + next_co as u32;
            if self.get(idx) == 0b11 {
                self.set(idx, self.depth % 3);
                self.to_visit.insert((i, next_co));
                self.new_at_depth += 1;
                self.num_entries += 1;
                if self.num_entries % 1_000_000 == 0 {
                    print!(
                        "\r{}M / {}M",
                        self.num_entries / 1_000_000,
                        self.total_entries / 1_000_000
                    );
                    let _ = std::io::stdout().lock().flush();
                }
            }
        }
    }
}
