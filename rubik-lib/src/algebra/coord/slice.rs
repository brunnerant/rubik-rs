use crate::{
    algebra::coord::{Coord, EO},
    core::state::State,
};

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
        lr.sample_state() * eo.sample_state()
    }
}
