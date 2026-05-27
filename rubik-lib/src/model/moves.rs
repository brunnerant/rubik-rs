use derive_more::derive::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
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
    pub fn axis_and_sign(&self) -> (Axis, bool) {
        match self {
            Face::Left => (Axis::X, false),
            Face::Right => (Axis::X, true),
            Face::Down => (Axis::Y, false),
            Face::Up => (Axis::Y, true),
            Face::Back => (Axis::Z, false),
            Face::Front => (Axis::Z, true),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Clockwise,
    CounterClockwise,
    HalfTurn,
}

impl Direction {
    pub fn inverse(&self) -> Self {
        match self {
            Direction::Clockwise => Direction::CounterClockwise,
            Direction::CounterClockwise => Direction::Clockwise,
            Direction::HalfTurn => Direction::HalfTurn,
        }
    }

    fn suffix(&self) -> &str {
        match self {
            Direction::Clockwise => "",
            Direction::CounterClockwise => "'",
            Direction::HalfTurn => "2",
        }
    }
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
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

    pub fn inverse(&self) -> Self {
        Self {
            face: self.face,
            dir: self.dir.inverse(),
        }
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

#[cfg(test)]
mod tests {
    use crate::model::moves::Move;

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
}
