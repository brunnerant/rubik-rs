
use crate::{
    algebra::{coord::Coord, sym::Symmetries, sym_coord::SymCoordTable},
    core::state::State,
};

pub struct RawCoordMoveTable<C: Coord> {
    move_table: Vec<C::Raw>,
}

impl<C: Coord> RawCoordMoveTable<C> {
    pub fn build() -> Self {
        let mut move_table = Vec::with_capacity(C::raw_to_usize(C::RAW_SIZE) * 18);
        for coord in C::all_raw_coords() {
            let s = C::from_repr(coord).sample_state();
            for i in 0..18 {
                let new_coord = C::from_state(&(s * State::BASIC_MOVES[i])).repr();
                move_table.push(new_coord);                
            }
        }
        Self { move_table }
    }

    pub fn coord_mv(&self, coord: C::Raw, mv: u8) -> C::Raw {
        self.move_table[18 * C::raw_to_usize(coord) + mv as usize]
    }
}

pub struct SymCoordMoveTable<C: Coord> {
    move_table: Vec<C::Raw>,
}

impl<C: Coord> SymCoordMoveTable<C> {
    pub fn build(sym_coord: &SymCoordTable<C>, sym: &Symmetries) -> Self {
        let mut move_table = Vec::with_capacity(C::sym_to_usize(C::SYM_SIZE) * 18);
        for coord in C::all_sym_coords() {
            let s = sym_coord.repr(coord);
            for i in 0..18 {
                move_table.push(sym_coord.sym_coord(s * State::BASIC_MOVES[i], sym));
            }
        }
        Self { move_table }
    }

    pub fn coord_mv(&self, coord: C::Raw, mv: u8, sym: &Symmetries) -> C::Raw {
        let (i, j) = C::unpack_sym_coord(coord);
        let mv2 = sym.conj_inv_mv(mv, j);
        let (k, l) = C::unpack_sym_coord(self.move_table[18 * C::sym_to_usize(i) + mv2 as usize]);
        C::pack_sym_coord(k, sym.prod(l, j))
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        algebra::{
            coord::{CO, Coord, EO, EOLR, LR},
            move_table::{RawCoordMoveTable, SymCoordMoveTable},
            sym::Symmetries,
            sym_coord::SymCoordTable,
        },
        core::{moves::Moves, state::State},
    };

    #[test]
    fn raw() {
        fn test<C: Coord>() {
            let move_table = RawCoordMoveTable::<C>::build();
            for (_, s) in Moves::to_depth(3) {
                let coord = C::from_state(&s).repr();
                for i in 0..18 {
                    let new_coord = move_table.coord_mv(coord, i);
                    let expected = C::from_state(&(s * State::BASIC_MOVES[i as usize])).repr();
                    assert_eq!(expected, new_coord);
                }
            }
        }

        test::<CO>();
        test::<EO>();
        test::<LR>();
        // test::<EOLR>(); quite big to compute
    }

    #[test]
    fn sym() {
        fn test<C: Coord>(sym: &Symmetries) {
            let sym_coord = SymCoordTable::<C>::build(sym);
            let move_table = SymCoordMoveTable::build(&sym_coord, sym);
            for (_, s) in Moves::to_depth(3) {
                let coord = sym_coord.sym_coord(s, sym);
                for i in 0..18 {
                    let new_coord = move_table.coord_mv(coord, i, sym);
                    let expected = sym_coord.sym_coord(s * State::BASIC_MOVES[i as usize], sym);
                    assert_eq!(expected, new_coord);
                }
            }
        }

        let sym = Symmetries::sub16();
        test::<CO>(&sym);
        test::<LR>(&sym);
        test::<EOLR>(&sym);
    }
}
