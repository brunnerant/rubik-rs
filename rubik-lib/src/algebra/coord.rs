use num::{Zero, iter::Range, range};
use std::fmt::Debug;

use crate::core::{bits::Int, state::State};

mod ori;
mod perm;
mod slice;

pub use ori::CO;
pub use ori::EO;
pub use perm::CP;
pub use perm::EP8;
pub use slice::EOLR;
pub use slice::LR;

/// A coordinate is a way to look at a subset of the full cube state.
/// For example, the orientation of the corners define a coordinate.
/// In terms of group theory, coordinates of the cube group G are defined
/// by cosets of a subgroup H. Elements in the same cosets have the same
/// coordinate.
pub trait Coord: Eq + Copy + Debug {
    /// The smallest bitfield that can contain this raw coordinate.
    type Raw: Int;
    /// The smallest bitfield that can contains this coordinate reduced by symmetry.
    /// Note that this type should always be smaller or equal to the raw type.
    type Sym: Int;

    /// The number of different values that this coordinate supports.
    const RAW_SIZE: Self::Raw;
    /// The number of different values that this coordinate supports when reduced by symmetry.
    const SYM_SIZE: Self::Sym;

    /// Builds a coordinate from a state
    fn from_state(state: &State) -> Self;
    /// Builds a coordinate from its raw representation
    fn from_repr(repr: Self::Raw) -> Self;

    /// Retrieves the representation of the coordinate
    fn repr(&self) -> Self::Raw;
    /// Sample a state that has this coordinate (there might be several such states).
    fn sample_state(&self) -> State;

    // Helper methods to convert between the coordinate types
    fn usize_to_sym(i: usize) -> Self::Sym {
        unsafe { num::cast::<usize, Self::Sym>(i).unwrap_unchecked() }
    }
    fn sym_to_usize(i: Self::Sym) -> usize {
        unsafe { num::cast::<Self::Sym, usize>(i).unwrap_unchecked() }
    }
    fn raw_to_usize(i: Self::Raw) -> usize {
        unsafe { num::cast::<Self::Raw, usize>(i).unwrap_unchecked() }
    }
    fn sym_to_raw(i: Self::Sym) -> Self::Raw {
        unsafe { num::cast::<Self::Sym, Self::Raw>(i).unwrap_unchecked() }
    }
    fn u8_to_raw(i: u8) -> Self::Raw {
        unsafe { num::cast::<u8, Self::Raw>(i).unwrap_unchecked() }
    }

    fn pack_sym_coord(s: Self::Sym, i: u8) -> Self::Raw {
        Self::sym_to_raw(s) * Self::u8_to_raw(16) + Self::u8_to_raw(i)
    }
    fn unpack_sym_coord(c: Self::Raw) -> (Self::Sym, u8) {
        let _16 = Self::u8_to_raw(16);
        let s = unsafe { num::cast::<Self::Raw, Self::Sym>(c / _16).unwrap_unchecked() };
        let i = unsafe { num::cast::<Self::Raw, u8>(c % _16).unwrap_unchecked() };
        (s, i)
    }

    // Coordinate iteration helpers
    fn all_raw_coords() -> Range<Self::Raw> {
        range(Zero::zero(), Self::RAW_SIZE)
    }
    fn all_sym_coords() -> Range<Self::Sym> {
        range(Zero::zero(), Self::SYM_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use itertools::Itertools;

    use crate::{
        algebra::{
            coord::{CO, CP, Coord, EO, EOLR, EP8, LR},
            sym::Symmetries,
        },
        core::{moves::Move, state::State},
    };

    #[test]
    fn sample_repr() {
        fn test<C: Coord>(closure: &[Move]) {
            let mut states = HashSet::new();
            for repr in C::all_raw_coords() {
                let coord = C::from_repr(repr);
                assert_eq!(coord.repr(), repr);
                let state = coord.sample_state();
                assert!(state.valid(), "{:?} {:?}", coord, state);
                assert_eq!(coord, C::from_state(&state), "{:?}", state);
                states.insert(state);

                // Check closure
                for &mv in closure {
                    let next_coord = C::from_state(&(mv * state));
                    assert!(next_coord.repr() < C::RAW_SIZE)
                }
            }
            assert_eq!(
                states.len(),
                num::cast::<C::Raw, usize>(C::RAW_SIZE).unwrap()
            );
        }

        const PHASE2_MOVES: [Move; 10] = [
            Move::L, Move::L_, Move::L2,
            Move::R, Move::R_, Move::R2,
            Move::D2, Move::U2, Move::B2, Move::F2,
        ];
        test::<CO>(&Move::BASIC_MOVES);
        test::<EO>(&Move::BASIC_MOVES);
        test::<CP>(&Move::BASIC_MOVES);
        test::<EP8>(&PHASE2_MOVES);
        test::<LR>(&Move::BASIC_MOVES);
        test::<EOLR>(&Move::BASIC_MOVES);
    }

    fn sym_invariant<C: Coord>(elems: &[State], sym: &Symmetries) -> bool {
        assert!(elems.iter().map(C::from_state).all_equal());
        for i in 0..sym.size() {
            if !elems
                .iter()
                .map(|&e| C::from_state(&sym.conj(e, i)))
                .all_equal()
            {
                return false;
            }
        }
        true
    }

    #[test]
    fn sym_invariance() {
        let full_sym = Symmetries::all();
        let red_sym = Symmetries::sub16();
        let elems = [State::ID, State::L];
        assert!(sym_invariant::<CO>(&elems, &red_sym));
        assert!(!sym_invariant::<CO>(&elems, &full_sym));
        assert!(sym_invariant::<EO>(&elems, &red_sym));
        assert!(!sym_invariant::<EO>(&elems, &full_sym));
        assert!(sym_invariant::<LR>(&elems, &red_sym));
        assert!(!sym_invariant::<LR>(&elems, &full_sym));
    }
}
