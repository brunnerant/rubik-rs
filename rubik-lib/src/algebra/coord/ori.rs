use crate::{algebra::coord::Coord, core::state::State};

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
