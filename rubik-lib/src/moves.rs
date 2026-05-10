use derive_more::derive::Display;

#[derive(Debug, Display)]
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

#[derive(Debug)]
pub enum Direction {
    Clockwise,
    CounterClockwise,
    HalfTurn,
}

impl Direction {
    fn suffix(&self) -> &str {
        match self {
            Direction::Clockwise => "",
            Direction::CounterClockwise => "'",
            Direction::HalfTurn => "2",
        }
    }
}

#[derive(Debug, Display)]
#[display("{face}{}", dir.suffix())]
pub struct Move {
    pub face: Face,
    pub dir: Direction,
}

impl Move {
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
    use crate::moves::Move;
    
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
