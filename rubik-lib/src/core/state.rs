use crate::core::{bits, moves::Move};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct State {
    /// Each corner contains its position (3 bits) and orientation (2 bits), compared to the solved cube.
    ///
    /// The positions tell where each corner moves when applying the state.
    /// Given cartesian (X, Y, Z) coordinates, they are defined as follows:
    /// - 0: (-1, -1, -1)
    /// - 1: (+1, -1, -1)
    /// - 2: (-1, +1, -1)
    /// - 3: (+1, +1, -1)
    /// - 4: (-1, -1, +1)
    /// - 5: (+1, -1, +1)
    /// - 6: (-1, +1, +1)
    /// - 7: (+1, +1, +1)
    ///
    /// The orientation of a corner tells how many times it was rotated clockwise from its neutral position.
    /// The neutral position is defined such that the color of the corner along the X axis (+X or -X depending on
    /// the position) is the same as the color of the adjacent center piece, considering opposite colors as identical.
    ///
    /// In order to support symmetries, the MSB of the corner field can be set to indicate a mirrored state.
    /// In such a state, corner orientations must be considered opposite to what they usually are,
    /// which alters state composition.
    pub corners: u64,
    /// Each edge contains its position (4 bits) and orientation (1 bit), compared to the solved cube.
    ///
    /// The positions tell where each edge moves when applying the state.
    /// Given cartesian (X, Y, Z) coordinates, they are defined as follows:
    /// - 0: (0, -1, -1)
    /// - 1: (0, +1, -1)
    /// - 2: (0, -1, +1)
    /// - 3: (0, +1, +1)
    /// - 4: (-1, 0, -1)
    /// - 5: (+1, 0, -1)
    /// - 6: (-1, 0, +1)
    /// - 7: (+1, 0, +1)
    /// - 8: (-1 -1, 0)
    /// - 9: (+1 -1, 0)
    /// - 10: (-1 +1, 0)
    /// - 11: (+1 +1, 0)
    ///
    /// The orientation of a corner tells whether it is in its neutral position or flipped.
    /// An edge is in neutral position if its dominant color is aligned with the adjacent dominant face.
    /// Dominance is induced by a simple X > Y > Z relation for ordering faces and corresponding colors.
    pub edges: u64,
}

impl State {
    /// The state of a solved cube.
    pub const ID: Self = Self {
        corners: 0b_11100_11000_10100_10000_01100_01000_00100_00000,
        edges: 0b_10110_10100_10010_10000_01110_01100_01010_01000_00110_00100_00010_00000,
    };

    /// The basic states resulting from the primary moves
    pub const BASIC_MOVES: [Self; 18] = [
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
    pub const L: Self = Self {
        corners: 0b_11100_01000_10100_11000_01100_00000_00100_10000,
        edges: 0b_10110_01000_10010_01100_01110_10100_01010_10000_00110_00100_00010_00000,
    };
    pub const L_: Self = Self {
        corners: 0b_11100_10000_10100_00000_01100_11000_00100_01000,
        edges: 0b_10110_01100_10010_01000_01110_10000_01010_10100_00110_00100_00010_00000,
    };
    pub const L2: Self = Self {
        corners: 0b_11100_00000_10100_01000_01100_10000_00100_11000,
        edges: 0b_10110_10000_10010_10100_01110_01000_01010_01100_00110_00100_00010_00000,
    };
    pub const R: Self = Self {
        corners: 0b_10100_11000_00100_10000_11100_01000_01100_00000,
        edges: 0b_01110_10100_01010_10000_10010_01100_10110_01000_00110_00100_00010_00000,
    };
    pub const R_: Self = Self {
        corners: 0b_01100_11000_11100_10000_00100_01000_10100_00000,
        edges: 0b_01010_10100_01110_10000_10110_01100_10010_01000_00110_00100_00010_00000,
    };
    pub const R2: Self = Self {
        corners: 0b_00100_11000_01100_10000_10100_01000_11100_00000,
        edges: 0b_10010_10100_10110_10000_01010_01100_01110_01000_00110_00100_00010_00000,
    };
    pub const D: Self = Self {
        corners: 0b_11100_11000_10010_00001_01100_01000_10101_00110,
        edges: 0b_10110_10100_00101_00001_01110_01100_01010_01000_00110_10001_00010_10011,
    };
    pub const D_: Self = Self {
        corners: 0b_11100_11000_00110_10101_01100_01000_00001_10010,
        edges: 0b_10110_10100_00001_00101_01110_01100_01010_01000_00110_10011_00010_10001,
    };
    pub const D2: Self = Self {
        corners: 0b_11100_11000_00000_00100_01100_01000_10000_10100,
        edges: 0b_10110_10100_10000_10010_01110_01100_01010_01000_00110_00000_00010_00100,
    };
    pub const U: Self = Self {
        corners: 0b_01101_11110_10100_10000_01010_11001_00100_00000,
        edges: 0b_00011_00111_10010_10000_01110_01100_01010_01000_10111_00100_10101_00000,
    };
    pub const U_: Self = Self {
        corners: 0b_11001_01010_10100_10000_11110_01101_00100_00000,
        edges: 0b_00111_00011_10010_10000_01110_01100_01010_01000_10101_00100_10111_00000,
    };
    pub const U2: Self = Self {
        corners: 0b_01000_01100_10100_10000_11000_11100_00100_00000,
        edges: 0b_10100_10110_10010_10000_01110_01100_01010_01000_00010_00100_00110_00000,
    };
    pub const B: Self = Self {
        corners: 0b_11100_11000_10100_10000_00101_01110_00010_01001,
        edges: 0b_10110_10100_10010_10000_01110_01100_00000_00010_00110_00100_01010_01000,
    };
    pub const B_: Self = Self {
        corners: 0b_11100_11000_10100_10000_01001_00010_01110_00101,
        edges: 0b_10110_10100_10010_10000_01110_01100_00010_00000_00110_00100_01000_01010,
    };
    pub const B2: Self = Self {
        corners: 0b_11100_11000_10100_10000_00000_00100_01000_01100,
        edges: 0b_10110_10100_10010_10000_01110_01100_01000_01010_00110_00100_00000_00010,
    };
    pub const F: Self = Self {
        corners: 0b_11010_10001_11101_10110_01100_01000_00100_00000,
        edges: 0b_10110_10100_10010_10000_00110_00100_01010_01000_01100_01110_00010_00000,
    };
    pub const F_: Self = Self {
        corners: 0b_10110_11101_10001_11010_01100_01000_00100_00000,
        edges: 0b_10110_10100_10010_10000_00100_00110_01010_01000_01110_01100_00010_00000,
    };
    pub const F2: Self = Self {
        corners: 0b_10000_10100_11000_11100_01100_01000_00100_00000,
        edges: 0b_10110_10100_10010_10000_01100_01110_01010_01000_00100_00110_00010_00000,
    };

    /// The corner permutation for corner i.
    pub fn cp(&self, i: u8) -> u8 {
        bits::get(self.corners, 5 * i + 2, 3) as u8
    }

    /// The corner orientation for corner i.
    pub fn co(&self, i: u8) -> u8 {
        bits::get(self.corners, 5 * i, 2) as u8
    }

    /// The edge permutation for edge i.
    pub fn ep(&self, i: u8) -> u8 {
        bits::get(self.edges, 5 * i + 1, 4) as u8
    }

    /// The edge orientation for edge i.
    pub fn eo(&self, i: u8) -> u8 {
        bits::get(self.edges, 5 * i, 1) as u8
    }

    /// Whether the state is mirrored, which means that orientations must be reversed.
    pub fn mirrored(&self) -> bool {
        (self.corners >> 63) != 0
    }

    /// Whether this state is valid. A state is valid if:
    /// - the edges and corners permutations are even
    /// - the corner orientations sum up to 0 mod 3
    /// - the edge orientations sum up to 0 mod 2
    pub fn valid(&self) -> bool {
        let mut corner_ori = 0;
        let mut corners = [0; 8];
        for i in 0..8 {
            let ori = self.co(i);
            if ori > 2 {
                return false;
            }
            corner_ori += ori;
            corners[i as usize] = self.cp(i);
        }
        let mut edge_ori = 0;
        let mut edges = [0; 12];
        for i in 0..12 {
            edge_ori += self.eo(i);
            edges[i as usize] = self.ep(i);
        }

        let mut even = true;
        corner_ori.is_multiple_of(3)
            && edge_ori.is_multiple_of(2)
            && Self::valid_perm(corners, &mut even)
            && Self::valid_perm(edges, &mut even)
            && even
    }

    fn valid_perm<const N: usize>(mut perm: [u8; N], even: &mut bool) -> bool {
        let mut present = [false; N];
        for i in 0..N {
            present[perm[i] as usize] = true;
        }
        if !present.into_iter().all(|p| p) {
            return false;
        }

        for i in 0..N {
            while perm[i] != i as u8 {
                let j = perm[i];
                perm.swap(i, j as usize);
                *even = !*even;
            }
        }

        true
    }

    /// Returns the inverse state.
    pub fn inv(&self) -> State {
        let mut corners = self.corners & (1 << 63);
        for i in 0..8 {
            let pos = self.cp(i);
            let ori = self.co(i) as u64;
            corners |= (i as u64) << (5 * pos + 2);
            corners |= ori << (5 * pos);
        }
        if !self.mirrored() {
            bits::bitwise_inv_mod_3(&mut corners);
        }

        let mut edges = 0;
        for i in 0..12 {
            let pos = self.ep(i);
            let ori = self.eo(i) as u64;
            edges |= (i as u64) << (5 * pos + 1);
            edges |= ori << (5 * pos);
        }

        State { corners, edges }
    }
}

impl std::ops::Mul for State {
    type Output = State;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut corners = (self.corners & (1 << 63)) ^ (rhs.corners & (1 << 63));
        for i in 0..8 {
            let pos = rhs.cp(i);
            let perm = bits::get(self.corners, 5 * pos, 5) as u8;
            corners |= (perm as u64) << (5 * i);
        }
        if rhs.mirrored() {
            bits::bitwise_inv_mod_3(&mut corners);
        }
        bits::bitwise_add_mod_3(&mut corners, rhs.corners);

        let mut edges = 0;
        for i in 0..12 {
            let pos = rhs.ep(i);
            let perm = bits::get(self.edges, 5 * pos, 5) as u8;
            edges |= (perm as u64) << (5 * i);
        }
        bits::bitwise_add_mod_2(&mut edges, rhs.edges);
        State { corners, edges }
    }
}

impl std::ops::Mul<Move> for State {
    type Output = State;

    fn mul(self, rhs: Move) -> Self::Output {
        let rhs: State = rhs.into();
        self * rhs
    }
}

impl std::ops::Mul<State> for Move {
    type Output = State;

    fn mul(self, rhs: State) -> Self::Output {
        let lhs: State = self.into();
        lhs * rhs
    }
}

impl std::ops::Mul<Move> for Move {
    type Output = State;

    fn mul(self, rhs: Move) -> Self::Output {
        let lhs: State = self.into();
        let rhs: State = rhs.into();
        lhs * rhs
    }
}

impl From<Move> for State {
    fn from(mv: Move) -> Self {
        State::BASIC_MOVES[mv.index() as usize]
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        write!(f, "corners")?;
        if self.mirrored() {
            write!(f, " (mirrored)")?;
        }
        writeln!(f, ":")?;
        write!(f, "- permutation:")?;
        for i in 0..8 {
            write!(f, " {}", self.cp(i))?;
        }
        writeln!(f)?;
        write!(f, "- orientation:")?;
        for i in 0..8 {
            write!(f, " {}", self.co(i))?;
        }
        writeln!(f)?;
        writeln!(f, "edges:")?;
        write!(f, "- permutation:")?;
        for i in 0..12 {
            write!(f, " {:02}", self.ep(i))?;
        }
        writeln!(f)?;
        write!(f, "- orientation:")?;
        for i in 0..12 {
            write!(f, "  {}", self.eo(i))?;
        }
        writeln!(f)?;
        Ok(())
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, hash::Hash};

    use itertools::iproduct;

    use crate::{
        core::moves::{Direction, Face, Move},
        core::state::State,
    };

    fn assert_distinct<T: Eq + Hash>(iter: impl Iterator<Item = T>) {
        let mut elems = HashSet::new();
        for elem in iter {
            if !elems.insert(elem) {
                panic!("elements are not distinct");
            }
        }
    }

    #[test]
    fn double_double_rotation_cancel() {
        for mv in [Move::L2, Move::R2, Move::D2, Move::U2, Move::B2, Move::F2] {
            assert_eq!(mv * mv, State::ID);
        }
    }

    #[test]
    fn all_moves_are_distinct() {
        assert_distinct(Move::BASIC_MOVES.iter().map(|&mv| (mv * State::ID).corners));
        assert_distinct(Move::BASIC_MOVES.iter().map(|&mv| (mv * State::ID).edges));
    }

    #[test]
    fn two_quarters_equal_half_turn() {
        for face in Face::ALL_FACES {
            let q = Move {
                face,
                dir: Direction::Clockwise,
            };
            let qi = q.inv();
            let h = Move {
                face,
                dir: Direction::HalfTurn,
            };
            assert_eq!(q * q, h.into());
            assert_eq!(qi * qi, h.into());
        }
    }

    #[test]
    fn four_quarters_cancel() {
        for face in Face::ALL_FACES {
            let q = Move {
                face,
                dir: Direction::Clockwise,
            };
            let qi = q.inv();
            assert_eq!(q * q * q * q, State::ID);
            assert_eq!(qi * qi * qi * qi, State::ID);
        }
    }

    #[test]
    fn state_inversion() {
        for (i, j) in iproduct!(0..18, 0..18) {
            let state = State::BASIC_MOVES[i] * State::BASIC_MOVES[j];
            let inv = state.inv();
            assert!(state.valid());
            assert!(inv.valid());
            assert_eq!(state * inv, State::ID);
            assert_eq!(inv * state, State::ID);
        }
    }

    #[test]
    fn state_idempotence() {
        assert!(State::ID.valid());
        for s in State::BASIC_MOVES {
            assert_eq!(s, s * State::ID);
            assert_eq!(s, State::ID * s);
        }
    }
}
