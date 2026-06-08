use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use crate::{
    algebra::{
        coord::{CO, EOLR},
        move_table::{RawCoordMoveTable, RawCoordSymTable, SymCoordMoveTable},
        sym::Symmetries,
        sym_coord::SymCoordTable,
    },
    core::io::BinarySerde,
};

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

    pub fn to_folder(&self, folder: impl AsRef<Path>) -> std::io::Result<()> {
        let folder = folder.as_ref();
        self.eolr_coord.to_file(folder.join("eolr-sym-coord.bin"))?;
        self.eolr_mv.to_file(folder.join("eolr-sym-coord-mv.bin"))?;
        self.co_mv.to_file(folder.join("co-raw-coord-mv.bin"))?;
        self.co_sym.to_file(folder.join("co-raw-coord-sym.bin"))
    }

    pub fn from_folder(folder: impl AsRef<Path>) -> std::io::Result<Self> {
        let folder = folder.as_ref();
        let eolr_coord = BinarySerde::from_file(folder.join("eolr-sym-coord.bin"))?;
        let eolr_mv = BinarySerde::from_file(folder.join("eolr-sym-coord-mv.bin"))?;
        let co_mv = BinarySerde::from_file(folder.join("co-raw-coord-mv.bin"))?;
        let co_sym = BinarySerde::from_file(folder.join("co-raw-coord-sym.bin"))?;
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
