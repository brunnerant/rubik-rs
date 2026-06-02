//! A coordinate is a way to look at a subset of the full cube state.
//! For example, the orientation of the corners define a coordinate.
//! In terms of group theory, coordinates of the cube group G are defined
//! by cosets of a subgroup H. Elements in the same cosets have the same
//! coordinate.

use std::fmt::Debug;

use crate::model::{bits::BitField, state::State};

pub trait Coord: Eq + Copy + Debug {
    /// The smallest bitfield that can contain this coordinate.
    type Repr: BitField;
    /// The number of different values that this coordinate supports.
    const COUNT: Self::Repr;

    /// Builds a coordinate from a state
    fn from_state(state: &State) -> Self;
    /// Builds a coordinate from its raw representation
    fn from_repr(repr: Self::Repr) -> Self;

    /// Retrieves the representation of the coordinate
    fn repr(&self) -> Self::Repr;
    /// Sample a state that has this coordinate (there might be several such states).
    fn sample_state(&self) -> State;
}

/// Corner orientation coordinate
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CO {
    coord: u16,
}

impl Coord for CO {
    type Repr = u16;
    const COUNT: Self::Repr = 2187; // 3^7

    fn from_state(state: &State) -> Self {
        let mut coord = 0;
        for i in (0..7).rev() {
            coord = coord * 3 + state.co(i) as u16;
        }
        Self { coord }
    }

    fn from_repr(repr: Self::Repr) -> Self {
        Self { coord: repr }
    }

    fn repr(&self) -> Self::Repr {
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
    type Repr = u16;
    const COUNT: Self::Repr = 2048; // 2^11

    fn from_state(state: &State) -> Self {
        let mut coord = 0;
        for i in 0..11 {
            coord |= (state.eo(i) as u16) << i;
        }
        Self { coord }
    }

    fn from_repr(repr: Self::Repr) -> Self {
        Self { coord: repr }
    }

    fn repr(&self) -> Self::Repr {
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
    type Repr = u16;
    const COUNT: Self::Repr = 495; // 12 choose 4

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

    fn from_repr(repr: Self::Repr) -> Self {
        Self { coord: repr }
    }

    fn repr(&self) -> Self::Repr {
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
    type Repr = u32;

    const COUNT: Self::Repr = EO::COUNT as u32 * LR::COUNT as u32;

    fn from_state(state: &State) -> Self {
        let eo = EO::from_state(state);
        let lr = LR::from_state(state);
        Self {
            coord: (eo.repr() as u32) * (LR::COUNT as u32) + lr.repr() as u32,
        }
    }

    fn from_repr(repr: Self::Repr) -> Self {
        Self { coord: repr }
    }

    fn repr(&self) -> Self::Repr {
        self.coord
    }

    fn sample_state(&self) -> State {
        let eo = EO::from_repr((self.coord / (LR::COUNT as u32)) as u16);
        let lr = LR::from_repr((self.coord % (LR::COUNT as u32)) as u16);
        lr.sample_state() * eo.sample_state()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use itertools::Itertools;
    use num::{Zero, range};

    use crate::model::{
        coord::{CO, Coord, EO, EOLR, LR},
        moves::Move,
        state::State,
        sym::Symmetries,
    };

    #[test]
    fn test_sample_repr() {
        fn test<C: Coord>() {
            let mut states = HashSet::new();
            for repr in range(Zero::zero(), C::COUNT) {
                let coord = C::from_repr(repr);
                assert_eq!(coord.repr(), repr);
                let state = coord.sample_state();
                assert!(state.valid(), "{:?} {:?}", coord, state);
                assert_eq!(coord, C::from_state(&state), "{:?}", state);
                states.insert(state);

                // Check closure
                for mv in Move::BASIC_MOVES {
                    let next_coord = C::from_state(&(mv * state));
                    assert!(next_coord.repr() < C::COUNT)
                }
            }
            assert_eq!(states.len(), num::cast::<C::Repr, usize>(C::COUNT).unwrap());
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
                .map(|&e| C::from_state(&sym.conj(e, i as u8)))
                .all_equal()
            {
                return false;
            }
        }
        true
    }

    #[test]
    fn test_sym_invariance() {
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
