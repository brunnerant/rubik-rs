use crate::{algebra::coord::Coord, core::state::State};

/// Corner orientation coordinate
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CO {
    coord: u16,
}

impl Coord for CO {
    type Raw = u16;
    type ReprIdx = u8;
    const NUM_RAW: Self::Raw = 2187; // 3^7
    const NUM_REPR: Self::ReprIdx = 168;

    fn from_state(state: &State) -> Self {
        let mut coord = 0;
        for i in (0..7).rev() {
            coord = coord * 3 + state.co(i) as u16;
        }
        Self { coord }
    }

    fn from_coord(repr: Self::Raw) -> Self {
        Self { coord: repr }
    }

    fn coord(&self) -> Self::Raw {
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
    type ReprIdx = u8;
    const NUM_RAW: Self::Raw = 2048; // 2^11
    const NUM_REPR: Self::ReprIdx = 186;

    fn from_state(state: &State) -> Self {
        let mut coord = 0;
        for i in 0..11 {
            coord |= (state.eo(i) as u16) << i;
        }
        Self { coord }
    }

    fn from_coord(repr: Self::Raw) -> Self {
        Self { coord: repr }
    }

    fn coord(&self) -> Self::Raw {
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
