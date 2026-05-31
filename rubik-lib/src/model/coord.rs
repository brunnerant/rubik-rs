//! A coordinate is a way to look at a subset of the full cube state.
//! For example, the orientation of the corners define a coordinate.
//! In terms of group theory, coordinates of the cube group G are defined
//! by cosets of a subgroup H. Elements in the same cosets have the same
//! coordinate.

use crate::model::{bits::BitField, state::State};

pub trait Coord {
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
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq)]
pub struct LR {
    coord: u16,
}

impl LR {
    const BINOM: [u16; 12 * 4] = [
        1, 0, 0, 0, 1, 1, 0, 0, 1, 2, 1, 0, 1, 3, 3, 1, 1, 4, 6, 4, 1, 5, 10, 10, 1, 6, 15, 20, 1,
        7, 21, 35, 1, 8, 28, 56, 1, 9, 36, 84, 1, 10, 45, 120, 1, 11, 55, 165,
    ];

    fn is_even(mut perm: [u8; 12]) -> bool {
        let mut even = true;
        for i in 0..12 {
            while perm[i] != i as u8 {
                let j = perm[i];
                perm.swap(i, j as usize);
                even = !even;
            }
        }
        even
    }
}

impl Coord for LR {
    type Repr = u16;
    const COUNT: Self::Repr = 495; // 12 choose 4

    fn from_state(state: &State) -> Self {
        let mut taken = [false; 12];
        taken[state.ep(0) as usize] = true;
        taken[state.ep(1) as usize] = true;
        taken[state.ep(2) as usize] = true;
        taken[state.ep(3) as usize] = true;

        let mut n = 12;
        let mut k = 4;
        let mut coord = 0;
        for tk in taken.into_iter() {
            if tk {
                k -= 1;
                if k == 0 {
                    break;
                }
            } else {
                coord += Self::BINOM[4 * (n - 1) + (k - 1)];
            }
            n -= 1;
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
        let mut taken = [false; 12];
        let mut n = 12;
        let mut k = 4;
        let mut coord = self.coord;
        loop {
            let binom = Self::BINOM[4 * (n - 1) + (k - 1)];
            if coord >= binom {
                coord -= binom;
            } else {
                taken[12 - n] = true;
                k -= 1;
                if k == 0 {
                    break;
                }
            }
            n -= 1;
        }

        let mut taken_i = 0;
        let mut not_taken_i = 4;
        let mut perm = [0; 12];
        for (i, tk) in taken.into_iter().enumerate() {
            if tk {
                perm[taken_i] = i as u8;
                taken_i += 1;
            } else {
                perm[not_taken_i] = i as u8;
                not_taken_i += 1;
            }
        }
        if !Self::is_even(perm) {
            perm.swap(0, 1);
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

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, fmt::Debug};

    use num::{Zero, range};

    use crate::model::coord::{CO, Coord, EO, LR};

    #[test]
    fn test_sample_repr() {
        fn test<C: Coord + Eq + Debug>()
        where
            C::Repr: std::fmt::Debug,
        {
            let mut states = HashSet::new();
            for repr in range(Zero::zero(), C::COUNT) {
                let coord = C::from_repr(repr);
                assert_eq!(coord.repr(), repr);
                let state = coord.sample_state();
                assert!(state.valid(), "{:?} {:?}", coord, state);
                assert_eq!(coord, C::from_state(&state), "{:?}", state);
                states.insert(state);
            }
            assert_eq!(states.len(), num::cast(C::COUNT).unwrap());
        }

        test::<CO>();
        test::<EO>();
        test::<LR>();
    }
}
