use crate::model::{
    bits,
    moves::{Axis, Direction, Face, Move},
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct State {
    /// Each corner contains its position (3 bits) and orientation (2 bits), compared to the solved cube.
    ///
    /// The positions tell where each corner comes from in the solved cube.
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
    pub corners: u64,
    /// Each edge contains its position (4 bits) and orientation (1 bit), compared to the solved cube.
    ///
    /// The positions tell where each edge comes from in the solved cube.
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
    /// Dominance is induced by a simple X < Y < Z relation for ordering faces and corresponding colors.
    pub edges: u64,
}

impl State {
    /// The state of a solved cube.
    pub const ID: Self = Self {
        corners: 0b_11100_11000_10100_10000_01100_01000_00100_00000,
        edges: 0b_10110_10100_10010_10000_01110_01100_01010_01000_00110_00100_00010_00000,
    };

    fn permute<const N: usize>(array: u64, perm: [u8; N], inv: bool) -> u64 {
        let mut result = 0;
        for (from, to) in perm.into_iter().enumerate() {
            let from = from as u8;
            let (from, to) = if inv { (to, from) } else { (from, to) };
            result |= bits::get(array, 5 * from, 5) << (5 * to);
        }
        result
    }

    fn orient_corners(array: u64, perm: [u8; 8], axis: u8, inv: bool) -> u64 {
        let mut result = 0;
        for (from, to) in perm.into_iter().enumerate() {
            // position
            let from = from as u8;
            let (from, to) = if inv { (to, from) } else { (from, to) };
            result |= bits::get(array, 5 * from + 2, 3) << (5 * to + 2);
            // orientation
            let offset = if axis != 0 && from != to {
                let dir = (from.count_ones() % 2 == 0) ^ (axis == 2);
                if dir { 1 } else { 2 }
            } else {
                0
            };
            let old_or = bits::get(array, 5 * from, 2);
            let new_or = (old_or + offset) % 3;
            result |= new_or << (5 * to);
        }
        result
    }

    fn orient_edges(array: u64, perm: [u8; 12], axis: u8, inv: bool) -> u64 {
        let mut result = 0;
        for (from, to) in perm.into_iter().enumerate() {
            // position
            let from = from as u8;
            let (from, to) = if inv { (to, from) } else { (from, to) };
            result |= bits::get(array, 5 * from + 1, 4) << (5 * to + 1);
            // orientation
            let flip = axis == 1 && from != to;
            result |= (flip as u64 ^ bits::get(array, 5 * from, 1)) << (5 * to)
        }
        result
    }

    /// Returns the state resulting from the given move.
    pub fn mv(&self, mv: Move) -> Self {
        match mv.dir {
            Direction::HalfTurn => {
                let corner_perm = match mv.face {
                    Face::Left => [6, 1, 4, 3, 2, 5, 0, 7],
                    Face::Right => [0, 7, 2, 5, 4, 3, 6, 1],
                    Face::Down => [5, 4, 2, 3, 1, 0, 6, 7],
                    Face::Up => [0, 1, 7, 6, 4, 5, 3, 2],
                    Face::Back => [3, 2, 1, 0, 4, 5, 6, 7],
                    Face::Front => [0, 1, 2, 3, 7, 6, 5, 4],
                };
                let edge_perm = match mv.face {
                    Face::Left => [0, 1, 2, 3, 6, 5, 4, 7, 10, 9, 8, 11],
                    Face::Right => [0, 1, 2, 3, 4, 7, 6, 5, 8, 11, 10, 9],
                    Face::Down => [2, 1, 0, 3, 4, 5, 6, 7, 9, 8, 10, 11],
                    Face::Up => [0, 3, 2, 1, 4, 5, 6, 7, 8, 9, 11, 10],
                    Face::Back => [1, 0, 2, 3, 5, 4, 6, 7, 8, 9, 10, 11],
                    Face::Front => [0, 1, 3, 2, 4, 5, 7, 6, 8, 9, 10, 11],
                };
                Self {
                    corners: Self::permute(self.corners, corner_perm, false),
                    edges: Self::permute(self.edges, edge_perm, false),
                }
            }
            _ => {
                let corner_perm = match mv.face {
                    Face::Left => [2, 1, 6, 3, 0, 5, 4, 7],
                    Face::Right => [0, 5, 2, 1, 4, 7, 6, 3],
                    Face::Down => [4, 0, 2, 3, 5, 1, 6, 7],
                    Face::Up => [0, 1, 3, 7, 4, 5, 2, 6],
                    Face::Back => [1, 3, 0, 2, 4, 5, 6, 7],
                    Face::Front => [0, 1, 2, 3, 6, 4, 7, 5],
                };
                let edge_perm = match mv.face {
                    Face::Left => [0, 1, 2, 3, 10, 5, 8, 7, 4, 9, 6, 11],
                    Face::Right => [0, 1, 2, 3, 4, 9, 6, 11, 8, 7, 10, 5],
                    Face::Down => [8, 1, 9, 3, 4, 5, 6, 7, 2, 0, 10, 11],
                    Face::Up => [0, 11, 2, 10, 4, 5, 6, 7, 8, 9, 1, 3],
                    Face::Back => [5, 4, 2, 3, 0, 1, 6, 7, 8, 9, 10, 11],
                    Face::Front => [0, 1, 6, 7, 4, 5, 3, 2, 8, 9, 10, 11],
                };
                let axis = match mv.face.axis() {
                    Axis::X => 0,
                    Axis::Y => 1,
                    Axis::Z => 2,
                };
                let inv = mv.dir == Direction::CounterClockwise;
                Self {
                    corners: Self::orient_corners(self.corners, corner_perm, axis, inv),
                    edges: Self::orient_edges(self.edges, edge_perm, axis, inv),
                }
            }
        }
    }

    /// Performs the given permutation after this permutation.
    pub fn then(&self, other: &State) -> State {
        let mut corners = 0;
        for i in 0..8 {
            let mut pos = i;
            let mut ori = bits::get(other.corners, 5 * pos, 2);
            pos = bits::get(other.corners, 5 * pos + 2, 3) as u8;
            ori = (ori + bits::get(self.corners, 5 * pos, 2)) % 3;
            pos = bits::get(self.corners, 5 * pos + 2, 3) as u8;
            corners |= (pos as u64) << (5 * i + 2);
            corners |= ori << (5 * i);
        }

        let mut edges = 0;
        for i in 0..12 {
            let mut pos = i;
            let mut ori = bits::get(other.edges, 5 * pos, 1);
            pos = bits::get(other.edges, 5 * pos + 1, 4) as u8;
            ori ^= bits::get(self.edges, 5 * pos, 1);
            pos = bits::get(self.edges, 5 * pos + 1, 4) as u8;
            edges |= (pos as u64) << (5 * i + 1);
            edges |= ori << (5 * i);
        }

        State { corners, edges }
    }

    pub fn inv(&self) -> State {
        let mut corners = 0;
        for i in 0..8 {
            let pos = bits::get(self.corners, 5 * i + 2, 3) as u8;
            let ori = bits::get(self.corners, 5 * i, 2);
            corners |= (i as u64) << (5 * pos + 2);
            corners |= ((3 - ori) % 3) << (5 * pos);
        }

        let mut edges = 0;
        for i in 0..12 {
            let pos = bits::get(self.edges, 5 * i + 1, 4) as u8;
            let ori = bits::get(self.edges, 5 * i, 1);
            edges |= (i as u64) << (5 * pos + 1);
            edges |= ori << (5 * pos);
        }

        State { corners, edges }
    }
}

impl std::ops::Mul for State {
    type Output = State;

    fn mul(self, rhs: Self) -> Self::Output {
        rhs.then(&self)
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f, "corners:")?;
        write!(f, "- permutation:")?;
        for i in 0..8 {
            write!(f, " {}", bits::get(self.corners, 5 * i + 2, 3))?;
        }
        writeln!(f)?;
        write!(f, "- orientation:")?;
        for i in 0..8 {
            write!(f, " {}", bits::get(self.corners, 5 * i, 2))?;
        }
        writeln!(f)?;
        writeln!(f, "edges:")?;
        write!(f, "- permutation:")?;
        for i in 0..12 {
            write!(f, " {:02}", bits::get(self.edges, 5 * i + 1, 4))?;
        }
        writeln!(f)?;
        write!(f, "- orientation:")?;
        for i in 0..12 {
            write!(f, "  {}", bits::get(self.edges, 5 * i, 1))?;
        }
        writeln!(f)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, hash::Hash};

    use itertools::iproduct;

    use crate::{
        model::moves::{Direction, Face, Move},
        model::state::State,
    };

    fn assert_distinct<T: Eq + Hash>(iter: impl Iterator<Item = T>) {
        let mut elems = HashSet::new();
        for elem in iter {
            if !elems.insert(elem) {
                assert!(false, "elements are not distinct");
            }
        }
    }

    #[test]
    fn double_double_rotation_cancel() {
        for mv in [Move::L2, Move::R2, Move::D2, Move::U2, Move::B2, Move::F2] {
            assert_eq!(State::ID.mv(mv).mv(mv), State::ID);
        }
    }

    #[test]
    fn all_moves_are_distinct() {
        assert_distinct(Move::BASIC_MOVES.iter().map(|&mv| State::ID.mv(mv).corners));
        assert_distinct(Move::BASIC_MOVES.iter().map(|&mv| State::ID.mv(mv).edges));
    }

    #[test]
    fn inverse_move_cancels() {
        for mv in Move::BASIC_MOVES {
            assert_eq!(State::ID.mv(mv).mv(mv.inv()), State::ID);
        }
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
            assert_eq!(State::ID.mv(q).mv(q), State::ID.mv(h));
            assert_eq!(State::ID.mv(qi).mv(qi), State::ID.mv(h));
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
            assert_eq!(State::ID.mv(q).mv(q).mv(q).mv(q), State::ID);
            assert_eq!(State::ID.mv(qi).mv(qi).mv(qi).mv(qi), State::ID);
        }
    }

    #[test]
    fn state_composition() {
        let basic_moves = Move::BASIC_MOVES;
        let basic_states = basic_moves.map(|m| State::ID.mv(m));

        for (i, j) in iproduct!(0..18, 0..18) {
            let state1 = State::ID.mv(basic_moves[i]).mv(basic_moves[j]);
            let state2 = basic_states[i].then(&basic_states[j]);
            let state3 = basic_states[j] * basic_states[i];
            assert_eq!(state1, state2, "{:?} != {:?}", state1, state2);
            assert_eq!(state1, state2, "{:?} != {:?}", state1, state3);
        }
    }

    #[test]
    fn state_inversion() {
        for (i, j) in iproduct!(0..18, 0..18) {
            let state = State::ID.mv(Move::BASIC_MOVES[i]).mv(Move::BASIC_MOVES[j]);
            let inv = state.inv();
            assert_eq!(state * inv, State::ID);
            assert_eq!(inv * state, State::ID);
        }
    }

    #[test]
    fn state_idempotence() {
        for mv in Move::BASIC_MOVES {
            let state = State::ID.mv(mv);
            assert_eq!(state, state * State::ID);
            assert_eq!(state, State::ID * state);
        }
    }
}
