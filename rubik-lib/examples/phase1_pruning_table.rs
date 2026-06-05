use rubik_lib::{algebra::{move_table::{RawCoordMoveTable, SymCoordMoveTable}, sym::Symmetries, sym_coord::SymCoordTable}, solve::kociemba};


fn main() {
    let sym = Symmetries::sub16();
    let eolr_coord = SymCoordTable::build(&sym);
    let eolr_mv = SymCoordMoveTable::build(&eolr_coord, &sym);
    let co_mv = RawCoordMoveTable::build();
    let pruning_table = kociemba::phase1::PruningTable::build(&eolr_coord, &eolr_mv, &co_mv, &sym);
}