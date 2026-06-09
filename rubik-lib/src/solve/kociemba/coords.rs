use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use crate::{
    algebra::{
        coord::{CO, CP, EOLR, EP4, EP8},
        move_table::{RawCoordMoveTable, RawCoordSymTable, SymCoordMoveTable},
        sym::Symmetries,
        sym_coord::SymCoordTable,
    },
    core::io::BinarySerde,
};

pub struct Coords {
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
}

impl Coords {
    pub fn build() -> Coords {
        let sym = Symmetries::sub16();
        let eolr_coord = SymCoordTable::build(&sym);
        let cp_coord = SymCoordTable::build(&sym);
        Self {
            eolr_mv: SymCoordMoveTable::build(&eolr_coord),
            eolr_coord,
            co_mv: RawCoordMoveTable::build(),
            co_sym: RawCoordSymTable::build(&sym),
            cp_mv: SymCoordMoveTable::build(&cp_coord),
            cp_coord,
            ep8_mv: RawCoordMoveTable::build(),
            ep8_sym: RawCoordSymTable::build(&sym),
            ep4_mv: RawCoordMoveTable::build(),
            sym,
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
        self.ep4_mv.to_file(folder.join("ep4-raw-coord-mv.bin"))
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
        })
    }

    pub fn buffer_from_file(file: &Path) -> std::io::Result<Vec<u8>> {
        let mut reader = BufReader::new(File::open(file)?);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}
