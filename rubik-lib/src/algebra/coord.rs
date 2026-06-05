//! A coordinate is a way to look at a subset of the full cube state.
//! For example, the orientation of the corners define a coordinate.
//! In terms of group theory, coordinates of the cube group G are defined
//! by cosets of a subgroup H. Elements in the same cosets have the same
//! coordinate.

use num::{Zero, iter::Range, range};
use std::fmt::Debug;

use crate::core::{bits::Int, state::State};

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

/// Corner orientation coordinate
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CO {
    coord: u16,
}

impl Coord for CO {
    type Raw = u16;
    type Sym = u8;
    const RAW_SIZE: Self::Raw = 2187; // 3^7
    const SYM_SIZE: Self::Sym = 168;

    fn from_state(state: &State) -> Self {
        let mut coord = 0;
        for i in (0..7).rev() {
            coord = coord * 3 + state.co(i) as u16;
        }
        Self { coord }
    }

    fn from_repr(repr: Self::Raw) -> Self {
        Self { coord: repr }
    }

    fn repr(&self) -> Self::Raw {
        self.coord
    }

    fn sample_state(&self) -> State {
        let mut coord = self.coord;
        let mut corners = State::ID.corners;
        let mut last_ori = 0;
        for i in 0..7 {
            let ori = coord % 3;
            corners |= (ori as u64) << (5 * i);
            last_ori += ori;
            coord /= 3;
        }
        corners |= (((300 - last_ori) % 3) as u64) << 35;
        State {
            corners,
            edges: State::ID.edges,
        }
    }
}

/// Edge orientation coordinate
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EO {
    coord: u16,
}

impl Coord for EO {
    type Raw = u16;
    type Sym = u8;
    const RAW_SIZE: Self::Raw = 2048; // 2^11
    const SYM_SIZE: Self::Sym = 186;

    fn from_state(state: &State) -> Self {
        let mut coord = 0;
        for i in 0..11 {
            coord |= (state.eo(i) as u16) << i;
        }
        Self { coord }
    }

    fn from_repr(repr: Self::Raw) -> Self {
        Self { coord: repr }
    }

    fn repr(&self) -> Self::Raw {
        self.coord
    }

    fn sample_state(&self) -> State {
        let mut edges = State::ID.edges;
        let mut last_ori = 0;
        for i in 0..11 {
            let ori = (self.coord >> i) & 1;
            edges |= (ori as u64) << (5 * i);
            last_ori ^= ori;
        }
        edges |= (last_ori as u64) << 55;
        State {
            corners: State::ID.corners,
            edges,
        }
    }
}

// LR slice edges coordinate
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LR {
    coord: u16,
}

impl LR {
    const BINOM: [u16; 12 * 5] = [
        0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 2, 1, 0, 0, 1, 3, 3, 1, 0, 1, 4, 6, 4, 0, 1, 5, 10, 10,
        0, 1, 6, 15, 20, 0, 1, 7, 21, 35, 0, 1, 8, 28, 56, 0, 1, 9, 36, 84, 0, 1, 10, 45, 120, 0,
        1, 11, 55, 165,
    ];
}

impl Coord for LR {
    type Raw = u16;
    type Sym = u8;
    const RAW_SIZE: Self::Raw = 495; // 12 choose 4
    const SYM_SIZE: Self::Sym = 45;

    fn from_state(state: &State) -> Self {
        let mut k = 4;
        let mut coord = 0;
        for i in 0..12 {
            if state.ep(i) < 4 {
                k -= 1;
            } else {
                coord += Self::BINOM[5 * (11 - i as usize) + k];
            }
        }
        Self { coord }
    }

    fn from_repr(repr: Self::Raw) -> Self {
        Self { coord: repr }
    }

    fn repr(&self) -> Self::Raw {
        self.coord
    }

    fn sample_state(&self) -> State {
        let mut k = 4;
        let mut coord = self.coord;
        let mut perm = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let mut even = true;
        let mut pos = [0; 4];
        for i in 0..12 {
            let binom = Self::BINOM[5 * (11 - i) + k];
            if coord >= binom {
                perm.swap(i, i + k);
                even ^= binom > 0;
                coord -= binom;
            } else {
                pos[4 - k] = i;
                k -= 1;
            }
        }
        if !even {
            perm.swap(pos[0], pos[1]);
        }

        let mut edges = 0;
        for (i, j) in perm.into_iter().enumerate() {
            edges |= (j as u64) << (5 * i + 1);
        }

        State {
            corners: State::ID.corners,
            edges,
        }
    }
}

/// Combined EO + LR coordinates
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EOLR {
    coord: u32,
}

impl Coord for EOLR {
    type Raw = u32;
    type Sym = u16;
    const RAW_SIZE: Self::Raw = EO::RAW_SIZE as u32 * LR::RAW_SIZE as u32;
    const SYM_SIZE: Self::Sym = 64430;

    fn from_state(state: &State) -> Self {
        let eo = EO::from_state(state);
        let lr = LR::from_state(state);
        Self {
            coord: (eo.repr() as u32) * (LR::RAW_SIZE as u32) + lr.repr() as u32,
        }
    }

    fn from_repr(repr: Self::Raw) -> Self {
        Self { coord: repr }
    }

    fn repr(&self) -> Self::Raw {
        self.coord
    }

    fn sample_state(&self) -> State {
        let eo = EO::from_repr((self.coord / (LR::RAW_SIZE as u32)) as u16);
        let lr = LR::from_repr((self.coord % (LR::RAW_SIZE as u32)) as u16);
        eo.sample_state() * lr.sample_state()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use itertools::Itertools;
    use num::{Zero, range};

    use crate::{
        algebra::{
            coord::{CO, Coord, EO, EOLR, LR},
            sym::Symmetries,
        },
        core::{moves::Move, state::State},
    };

    #[test]
    fn sample_repr() {
        fn test<C: Coord>() {
            let mut states = HashSet::new();
            for repr in range(Zero::zero(), C::RAW_SIZE) {
                let coord = C::from_repr(repr);
                assert_eq!(coord.repr(), repr);
                let state = coord.sample_state();
                assert!(state.valid(), "{:?} {:?}", coord, state);
                assert_eq!(coord, C::from_state(&state), "{:?}", state);
                states.insert(state);

                // Check closure
                for mv in Move::BASIC_MOVES {
                    let next_coord = C::from_state(&(mv * state));
                    assert!(next_coord.repr() < C::RAW_SIZE)
                }
            }
            assert_eq!(
                states.len(),
                num::cast::<C::Raw, usize>(C::RAW_SIZE).unwrap()
            );
        }

        test::<CO>();
        test::<EO>();
        test::<LR>();
        test::<EOLR>();
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
