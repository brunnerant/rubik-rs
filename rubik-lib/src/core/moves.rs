use std::collections::VecDeque;

use derive_more::derive::Display;
use smallvec::{SmallVec, smallvec};

use crate::core::state::State;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    #[display("L")]
    Left,
    #[display("R")]
    Right,
    #[display("D")]
    Down,
    #[display("U")]
    Up,
    #[display("B")]
    Back,
    #[display("F")]
    Front,
}

#[derive(PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    /// Returns the negative and positive faces along the axis, respectively.
    pub fn faces(&self) -> (Face, Face) {
        match self {
            Axis::X => (Face::Left, Face::Right),
            Axis::Y => (Face::Down, Face::Up),
            Axis::Z => (Face::Back, Face::Front),
        }
    }

    pub const fn index(&self) -> u8 {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
}

impl Face {
    pub const ALL_FACES: [Face; 6] = [
        Face::Left,
        Face::Right,
        Face::Down,
        Face::Up,
        Face::Back,
        Face::Front,
    ];

    /// Returns the axis, and whether the face is positively oriented along that axis.
    pub const fn axis(&self) -> Axis {
        match self {
            Face::Left => Axis::X,
            Face::Right => Axis::X,
            Face::Down => Axis::Y,
            Face::Up => Axis::Y,
            Face::Back => Axis::Z,
            Face::Front => Axis::Z,
        }
    }

    pub const fn index(&self) -> u8 {
        match self {
            Face::Left => 0,
            Face::Right => 1,
            Face::Down => 2,
            Face::Up => 3,
            Face::Back => 4,
            Face::Front => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Clockwise,
    CounterClockwise,
    HalfTurn,
}

impl Direction {
    pub const fn inv(&self) -> Self {
        match self {
            Direction::Clockwise => Direction::CounterClockwise,
            Direction::CounterClockwise => Direction::Clockwise,
            Direction::HalfTurn => Direction::HalfTurn,
        }
    }

    pub const fn suffix(&self) -> &str {
        match self {
            Direction::Clockwise => "",
            Direction::CounterClockwise => "'",
            Direction::HalfTurn => "2",
        }
    }

    pub const fn index(&self) -> u8 {
        match self {
            Direction::Clockwise => 0,
            Direction::CounterClockwise => 1,
            Direction::HalfTurn => 2,
        }
    }
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash)]
#[display("{face}{}", dir.suffix())]
pub struct Move {
    pub face: Face,
    pub dir: Direction,
}

impl Move {
    pub const BASIC_MOVES: [Move; 18] = [
        Self::L,
        Self::L_,
        Self::L2,
        Self::R,
        Self::R_,
        Self::R2,
        Self::D,
        Self::D_,
        Self::D2,
        Self::U,
        Self::U_,
        Self::U2,
        Self::B,
        Self::B_,
        Self::B2,
        Self::F,
        Self::F_,
        Self::F2,
    ];

    pub const fn inv(&self) -> Self {
        Self {
            face: self.face,
            dir: self.dir.inv(),
        }
    }

    pub const fn index(&self) -> u8 {
        self.face.index() * 3 + self.dir.index()
    }

    pub const L: Move = Move {
        face: Face::Left,
        dir: Direction::Clockwise,
    };
    pub const L_: Move = Move {
        face: Face::Left,
        dir: Direction::CounterClockwise,
    };
    pub const L2: Move = Move {
        face: Face::Left,
        dir: Direction::HalfTurn,
    };
    pub const R: Move = Move {
        face: Face::Right,
        dir: Direction::Clockwise,
    };
    pub const R_: Move = Move {
        face: Face::Right,
        dir: Direction::CounterClockwise,
    };
    pub const R2: Move = Move {
        face: Face::Right,
        dir: Direction::HalfTurn,
    };
    pub const D: Move = Move {
        face: Face::Down,
        dir: Direction::Clockwise,
    };
    pub const D_: Move = Move {
        face: Face::Down,
        dir: Direction::CounterClockwise,
    };
    pub const D2: Move = Move {
        face: Face::Down,
        dir: Direction::HalfTurn,
    };
    pub const U: Move = Move {
        face: Face::Up,
        dir: Direction::Clockwise,
    };
    pub const U_: Move = Move {
        face: Face::Up,
        dir: Direction::CounterClockwise,
    };
    pub const U2: Move = Move {
        face: Face::Up,
        dir: Direction::HalfTurn,
    };
    pub const B: Move = Move {
        face: Face::Back,
        dir: Direction::Clockwise,
    };
    pub const B_: Move = Move {
        face: Face::Back,
        dir: Direction::CounterClockwise,
    };
    pub const B2: Move = Move {
        face: Face::Back,
        dir: Direction::HalfTurn,
    };
    pub const F: Move = Move {
        face: Face::Front,
        dir: Direction::Clockwise,
    };
    pub const F_: Move = Move {
        face: Face::Front,
        dir: Direction::CounterClockwise,
    };
    pub const F2: Move = Move {
        face: Face::Front,
        dir: Direction::HalfTurn,
    };
}

/// An encoded sequence of basic moves.
/// It is encoded using a power series with move indices.
/// Due to the chosen size of the encoding, the sequences can
/// be of length at most 30, which should be enough for
/// most cases.
pub struct Moves {
    encoded: u128,
}

impl Moves {
    const EMPTY: Moves = Self { encoded: 0 };

    pub fn from_move(mv: Move) -> Self {
        Self {
            encoded: mv.index() as u128 + 1,
        }
    }

    pub fn from_moves(moves: impl IntoIterator<Item = Move>) -> Self {
        let mut result = Self::EMPTY;
        let mut count = 0;
        for mv in moves {
            result = result.append(mv);
            count += 1;
        }
        assert!(
            count <= 30,
            "cannot encode sequences with more than 30 moves"
        );
        result
    }

    pub fn append(&self, mv: Move) -> Self {
        Self {
            encoded: 19 * self.encoded + mv.index() as u128 + 1,
        }
    }

    pub fn moves(&self) -> SmallVec<[Move; 30]> {
        let mut result = smallvec![];
        let mut encoding = self.encoded;
        while encoding > 0 {
            result.push(Move::BASIC_MOVES[((encoding % 19) - 1) as usize]);
            encoding /= 19;
        }
        result.reverse();
        result
    }

    pub fn inv_moves(&self) -> SmallVec<[Move; 30]> {
        let mut result = smallvec![];
        let mut encoding = self.encoded;
        while encoding > 0 {
            result.push(Move::BASIC_MOVES[((encoding % 19) - 1) as usize].inv());
            encoding /= 19;
        }
        result
    }

    pub fn to_depth(d: u8) -> MovesIter {
        MovesIter {
            next: VecDeque::from([(Moves::EMPTY, State::ID)]),
            max: 19_u128.pow(d as u32),
        }
    }
}

pub struct MovesIter {
    next: VecDeque<(Moves, State)>,
    max: u128,
}

impl Iterator for MovesIter {
    type Item = (Moves, State);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.pop_front().map(|(moves, state)| {
            if 19 * moves.encoded < self.max {
                for mv in Move::BASIC_MOVES {
                    self.next.push_back((moves.append(mv), mv * state));
                }
            }
            (moves, state)
        })
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use smallvec::{SmallVec, smallvec};

    use crate::core::{
        moves::{Move, Moves},
        state::State,
    };

    #[test]
    fn move_display() {
        assert_eq!(Move::L.to_string(), "L");
        assert_eq!(Move::R.to_string(), "R");
        assert_eq!(Move::D.to_string(), "D");
        assert_eq!(Move::U.to_string(), "U");
        assert_eq!(Move::B.to_string(), "B");
        assert_eq!(Move::F.to_string(), "F");
        assert_eq!(Move::F_.to_string(), "F'");
        assert_eq!(Move::F2.to_string(), "F2");
    }

    #[test]
    fn move_index() {
        for i in 0..Move::BASIC_MOVES.len() {
            assert_eq!(i as u8, Move::BASIC_MOVES[i].index());
        }
    }

    #[test]
    fn unique_moves_to_depth_4() {
        assert_eq!(Moves::to_depth(4).unique_by(|&(_, s)| s).count(), 46741);
    }

    #[test]
    fn from_moves() {
        let moves_orig: SmallVec<[_; 15]> = smallvec![Move::F, Move::B_, Move::L];
        let moves = Moves::from_moves(moves_orig.clone());
        assert_eq!(moves_orig, moves.moves());
    }

    #[test]
    fn inv_moves() {
        let moves = Moves::from_moves([Move::F, Move::B_, Move::L]);
        let state = moves
            .moves()
            .into_iter()
            .fold(State::ID, |state, mv| mv * state);
        let inv_state = moves
            .inv_moves()
            .into_iter()
            .fold(State::ID, |state, mv| mv * state);
        assert_eq!(state.inv(), inv_state);
    }
}
