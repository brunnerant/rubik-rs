use crate::{
    algebra::coord::Coord,
    core::{
        bits::{self},
        state::State,
    },
};

/// Corner permutation coordinate
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CP {
    coord: u16,
}

impl Coord for CP {
    type Raw = u16;
    type ReprIdx = u16;
    const NUM_RAW: Self::Raw = 40320; // 8!
    const NUM_REPR: Self::ReprIdx = 2768;

    fn from_state(state: &State) -> Self {
        Self {
            coord: perm_to_coord::<8>(state.corners >> 2, 3),
        }
    }

    fn from_coord(repr: Self::Raw) -> Self {
        Self { coord: repr }
    }

    fn coord(&self) -> Self::Raw {
        self.coord
    }

    fn sample_state(&self) -> State {
        let (mut corners, even) = coord_to_perm::<8>(self.coord);
        corners <<= 2;
        let edges = if even {
            State::ID.edges
        } else {
            0b_10110_10100_10010_10000_01110_01100_01010_01000_00110_00100_00000_00010
        };
        State { corners, edges }
    }
}

/// Edge permutation coordinate for the edges belonging to faces L and R in phase 2
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EP8 {
    coord: u16,
}

impl Coord for EP8 {
    type Raw = u16;
    type ReprIdx = u16;
    const NUM_RAW: Self::Raw = 40320; // 8!
    const NUM_REPR: Self::ReprIdx = 2768;

    fn from_state(state: &State) -> Self {
        let edges = (state.edges >> 21) - 0b_00100_00100_00100_00100_00100_00100_00100_00100;
        Self {
            coord: perm_to_coord::<8>(edges, 4),
        }
    }

    fn from_coord(repr: Self::Raw) -> Self {
        Self { coord: repr }
    }

    fn coord(&self) -> Self::Raw {
        self.coord
    }

    fn sample_state(&self) -> State {
        let (mut edges, even) = coord_to_perm::<8>(self.coord);
        edges += 0b_00100_00100_00100_00100_00100_00100_00100_00100;
        edges <<= 21;
        if even {
            edges |= 0b_00110_00100_00010_00000;
        } else {
            edges |= 0b_00110_00100_00000_00010;
        };
        State {
            corners: State::ID.corners,
            edges,
        }
    }
}

/// Edge permutation coordinate for the LR slice edges in phase 2
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EP4 {
    coord: u8,
}

impl Coord for EP4 {
    type Raw = u8;
    type ReprIdx = u8;
    const NUM_RAW: Self::Raw = 24; // 4!
    const NUM_REPR: Self::ReprIdx = 24;

    fn from_state(state: &State) -> Self {
        Self {
            coord: perm_to_coord::<4>(state.edges >> 1, 4) as u8,
        }
    }

    fn from_coord(repr: Self::Raw) -> Self {
        Self { coord: repr }
    }

    fn coord(&self) -> Self::Raw {
        self.coord
    }

    fn sample_state(&self) -> State {
        let (mut edges, even) = coord_to_perm::<4>(self.coord as u16);
        edges <<= 1;
        if even {
            edges |= 0b_10110_10100_10010_10000_01110_01100_01010_01000 << 20;
        } else {
            edges |= 0b_10110_10100_10010_10000_01110_01100_01000_01010 << 20;
        };
        State {
            corners: State::ID.corners,
            edges,
        }
    }
}

fn perm_to_coord<const N: usize>(perm: u64, k: u8) -> u16 {
    let mut coord = 0;
    let mut left_set: u8 = 0;
    for i in 0..N as u8 - 1 {
        let cpi = bits::get(perm, 5 * i, k);
        let num_smaller = (left_set & ((1 << cpi) - 1)).count_ones();
        coord *= N as u16 - i as u16;
        coord += cpi as u16 - num_smaller as u16;
        left_set |= 1 << cpi;
    }
    coord
}

fn coord_to_perm<const N: usize>(mut coord: u16) -> (u64, bool) {
    let mut digits = [0; N];
    for i in 0..N as u16 - 1 {
        let k = i + 2;
        digits[N - 2 - i as usize] = coord % k;
        coord /= k;
    }
    let mut avail = [0, 1, 2, 3, 4, 5, 6, 7];
    let mut corners = 0;
    let mut even = true;
    for (i, d) in digits.into_iter().enumerate() {
        let pos = d as usize;
        corners |= avail[pos] << (5 * i);
        for j in pos..N - 1 - i {
            avail[j] = avail[j + 1];
        }
        even ^= !(N - 1 - i - pos).is_multiple_of(2);
    }
    (corners | (avail[0] << 35), even)
}
