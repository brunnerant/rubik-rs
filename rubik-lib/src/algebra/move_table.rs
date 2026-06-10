use num::traits::FromBytes;

use crate::{
    algebra::{coord::Coord, sym::Symmetries, sym_coord::SymCoordTable},
    core::{
        bits::{deserialize_array, serialize_array},
        io::BinarySerde,
        state::State,
    },
};

pub struct RawCoordMoveTable<C: Coord> {
    move_table: Vec<C::Raw>,
}

impl<C: Coord> RawCoordMoveTable<C> {
    pub fn build() -> Self {
        let mut move_table = Vec::with_capacity(C::raw_to_usize(C::NUM_RAW) * 18);
        for coord in C::raw_coords() {
            let s = C::from_coord(coord).sample_state();
            for i in 0..18 {
                let new_coord = C::from_state(&(s * State::BASIC_MOVES[i])).coord();
                move_table.push(new_coord);
            }
        }
        Self { move_table }
    }

    pub fn coord_mv(&self, coord: C::Raw, mv: u8) -> C::Raw {
        self.move_table[18 * C::raw_to_usize(coord) + mv as usize]
    }
}

impl<C: Coord> BinarySerde for RawCoordMoveTable<C>
where
    for<'a> &'a [u8]: TryInto<&'a <C::Raw as FromBytes>::Bytes>,
{
    fn to_binary(&self) -> Vec<u8> {
        serialize_array(&self.move_table)
    }

    fn from_binary(buffer: &[u8]) -> Option<Self> {
        let move_table = deserialize_array(buffer);
        (move_table.len() == C::raw_to_usize(C::NUM_RAW) * 18).then_some(Self { move_table })
    }
}

pub struct SymCoordMoveTable<C: Coord> {
    move_table: Vec<C::Raw>,
}

impl<C: Coord> SymCoordMoveTable<C> {
    pub fn build(sym_coord: &SymCoordTable<C>) -> Self {
        let mut move_table = Vec::with_capacity(C::sym_to_usize(C::NUM_REPR) * 18);
        for coord in C::repr_indices() {
            let repr = sym_coord.repr(coord);
            let state = C::from_coord(repr).sample_state();
            for i in 0..18 {
                let raw = C::from_state(&(state * State::BASIC_MOVES[i])).coord();
                move_table.push(sym_coord.raw_to_sym(raw));
            }
        }
        Self { move_table }
    }

    pub fn repr_mv(&self, repr: C::ReprIdx, mv: u8) -> C::Raw {
        self.move_table[18 * C::sym_to_usize(repr) + mv as usize]
    }

    pub fn coord_mv(&self, coord: C::Raw, mv: u8, sym: &Symmetries) -> C::Raw {
        let (i, j) = C::unpack_sym_coord(coord);
        let mv2 = sym.conj_inv_mv(mv, j);
        let (k, l) = C::unpack_sym_coord(self.repr_mv(i, mv2));
        C::pack_sym_coord(k, sym.prod(l, j))
    }
}

impl<C: Coord> BinarySerde for SymCoordMoveTable<C>
where
    for<'a> &'a [u8]: TryInto<&'a <C::Raw as FromBytes>::Bytes>,
{
    fn to_binary(&self) -> Vec<u8> {
        serialize_array(&self.move_table)
    }

    fn from_binary(buffer: &[u8]) -> Option<Self> {
        let move_table = deserialize_array(buffer);
        (move_table.len() == C::sym_to_usize(C::NUM_REPR) * 18).then_some(Self { move_table })
    }
}

pub struct RawCoordSymTable<C: Coord> {
    sym_table: Vec<C::Raw>,
    sym_size: usize,
}

impl<C: Coord> RawCoordSymTable<C> {
    pub fn build(sym: &Symmetries) -> Self {
        let sym_size = sym.size() as usize;
        let mut sym_table = Vec::with_capacity(C::raw_to_usize(C::NUM_RAW) * sym_size);
        for coord in C::raw_coords() {
            let s = C::from_coord(coord).sample_state();
            for i in 0..sym.size() {
                let new_coord = C::from_state(&sym.conj(s, i)).coord();
                sym_table.push(new_coord);
            }
        }
        Self {
            sym_table,
            sym_size,
        }
    }

    pub fn coord_sym(&self, coord: C::Raw, s: u8) -> C::Raw {
        self.sym_table[C::raw_to_usize(coord) * self.sym_size + s as usize]
    }
}

impl<C: Coord> BinarySerde for RawCoordSymTable<C>
where
    for<'a> &'a [u8]: TryInto<&'a <C::Raw as FromBytes>::Bytes>,
{
    fn to_binary(&self) -> Vec<u8> {
        serialize_array(&self.sym_table)
    }

    fn from_binary(buffer: &[u8]) -> Option<Self> {
        let sym_table = deserialize_array(buffer);
        let sym_size = sym_table.len() / C::raw_to_usize(C::NUM_RAW);
        Some(Self {
            sym_table,
            sym_size,
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        algebra::{
            coord::{CO, Coord, EO, EOLR, LR},
            move_table::{RawCoordMoveTable, RawCoordSymTable, SymCoordMoveTable},
            sym::Symmetries,
            sym_coord::SymCoordTable,
        },
        core::{moves::Moves, state::State},
    };

    #[test]
    fn raw_mv() {
        fn test<C: Coord>() {
            let move_table = RawCoordMoveTable::<C>::build();
            for (_, s) in Moves::to_depth(3) {
                let coord = C::from_state(&s).coord();
                for i in 0..18 {
                    let new_coord = move_table.coord_mv(coord, i);
                    let expected = C::from_state(&(s * State::BASIC_MOVES[i as usize])).coord();
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
    fn sym_mv() {
        fn test<C: Coord>(sym: &Symmetries) {
            let sym_coord = SymCoordTable::<C>::build(sym);
            let move_table = SymCoordMoveTable::build(&sym_coord);
            let sym_table = RawCoordSymTable::build(sym);
            for (_, s) in Moves::to_depth(3) {
                let raw = C::from_state(&s).coord();
                let coord = sym_coord.raw_to_sym(raw);
                for i in 0..18 {
                    let actual = move_table.coord_mv(coord, i, sym);
                    let new_raw = C::from_state(&(s * State::BASIC_MOVES[i as usize])).coord();
                    let expected = sym_coord.raw_to_sym(new_raw);

                    // Here there is a tricky bit:
                    // It is possible that the two sym coords are different, but that they map to the same
                    // raw coordinate. Therefore, we must compare the raw coordinates in order to test the move table.
                    // This can happen because two sym coords (i, j1) and (i, j2) map to the same raw coordinate due
                    // to internal symmetries of the corresponding raw coordinate.
                    let actual_raw = sym_coord.sym_to_raw(actual, &sym_table);
                    let expected_raw = sym_coord.sym_to_raw(expected, &sym_table);
                    assert_eq!(actual_raw, expected_raw);
                }
            }
        }

        let sym = Symmetries::sub16();
        test::<CO>(&sym);
        test::<LR>(&sym);
        test::<EOLR>(&sym);
    }

    #[test]
    fn raw_sym() {
        fn test<C: Coord>(sym: &Symmetries) {
            let sym_table = RawCoordSymTable::<C>::build(sym);
            for (_, s) in Moves::to_depth(3) {
                let coord = C::from_state(&s).coord();
                for i in 0..sym.size() {
                    let new_s = sym.conj(s, i);
                    let new_coord_expected = C::from_state(&new_s).coord();
                    let new_coord_actual = sym_table.coord_sym(coord, i);
                    assert_eq!(new_coord_expected, new_coord_actual);
                }
            }
        }

        let sym = Symmetries::sub16();
        test::<CO>(&sym);
        test::<LR>(&sym);
        test::<EOLR>(&sym);
    }
}
