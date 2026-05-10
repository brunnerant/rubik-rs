use bitvec::prelude::*;

use crate::moves::{Direction, Face, Move};

type BitArray = bitvec::array::BitArray<[u64; 1]>;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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
    pub corners: BitArray,
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
    /// To find the neutral position of an edge, consider its two colors and compare it to the two colors of the adjacent center pieces,
    /// treating opposite colors as identical. Take the common color between those two pairs, and orient the edge such that the color is
    /// adjacent to the corresponding center piece. If two colors are common between the edge and the center pieces, favor X over Y over Z.
    pub edges: BitArray,
}

impl State {
    /// The state of a solved cube.
    pub const SOLVED: State = State {
        corners: bitarr![const u64, Lsb0;
            0, 0, 0,        0, 0,
            1, 0, 0,        0, 0,
            0, 1, 0,        0, 0,
            1, 1, 0,        0, 0,
            0, 0, 1,        0, 0,
            1, 0, 1,        0, 0,
            0, 1, 1,        0, 0,
            1, 1, 1,        0, 0,
        ],
        edges: bitarr![const u64, Lsb0;
            0, 0, 0, 0,     0,
            1, 0, 0, 0,     0,
            0, 1, 0, 0,     0,
            1, 1, 0, 0,     0,
            0, 0, 1, 0,     0,
            1, 0, 1, 0,     0,
            0, 1, 1, 0,     0,
            1, 1, 1, 0,     0,
            0, 0, 0, 1,     0,
            1, 0, 0, 1,     0,
            0, 1, 0, 1,     0,
            1, 1, 0, 1,     0,
        ],
    };

    fn permute<const N: usize>(array: &BitArray, perm: [usize; N]) -> BitArray {
        let mut result = BitArray::new([0]);
        for (from, to) in perm.into_iter().enumerate() {
            result[5 * to..5 * (to + 1)].copy_from_bitslice(&array[5 * from..5 * (from + 1)]);
        }
        result
    }

    /// Returns the state resulting from the given move.
    pub fn mv(&self, mv: Move) -> State {
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
                State {
                    corners: Self::permute(&self.corners, corner_perm),
                    edges: Self::permute(&self.edges, edge_perm),
                }
            }
            Direction::Clockwise => {
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
                State {
                    corners: Self::permute(&self.corners, corner_perm),
                    edges: Self::permute(&self.edges, edge_perm),
                }
            },
            Direction::CounterClockwise => {
                let corner_perm = match mv.face {
                    Face::Left => [4, 1, 0, 3, 6, 5, 2, 7],
                    Face::Right => [0, 3, 2, 7, 4, 1, 6, 5],
                    Face::Down => [1, 5, 2, 3, 0, 4, 6, 7],
                    Face::Up => [0, 1, 6, 2, 4, 5, 7, 3],
                    Face::Back => [2, 0, 3, 1, 4, 5, 6, 7],
                    Face::Front => [0, 1, 2, 3, 5, 7, 4, 6],
                };
                let edge_perm = match mv.face {
                    Face::Left => [0, 1, 2, 3, 8, 5, 10, 7, 6, 9, 4, 11],
                    Face::Right => [0, 1, 2, 3, 4, 11, 6, 9, 8, 5, 10, 7],
                    Face::Down => [9, 1, 8, 3, 4, 5, 6, 7, 0, 2, 10, 11],
                    Face::Up => [0, 10, 2, 11, 4, 5, 6, 7, 8, 9, 3, 1],
                    Face::Back => [4, 5, 2, 3, 1, 0, 6, 7, 8, 9, 10, 11],
                    Face::Front => [0, 1, 7, 6, 4, 5, 2, 3, 8, 9, 10, 11],
                };
                State {
                    corners: Self::permute(&self.corners, corner_perm),
                    edges: Self::permute(&self.edges, edge_perm),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, hash::Hash};

    use crate::{moves::{Direction, Move}, state::State};

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
            assert_eq!(State::SOLVED.mv(mv).mv(mv), State::SOLVED);
        }
    }

    #[test]
    fn double_rotations_are_distinct() {
        assert_distinct(
            [Move::L2, Move::R2, Move::D2, Move::U2, Move::B2, Move::F2]
                .into_iter()
                .map(|mv| State::SOLVED.mv(mv)),
        );
    }

    #[test]
    fn forward_cancels_backward() {
        for mv in [Move::L, Move::R, Move::D, Move::U, Move::B, Move::F] {
            assert_eq!(mv.inverse().face, mv.face);
            assert_eq!(mv.dir, Direction::Clockwise);
            assert_eq!(mv.inverse().dir, Direction::CounterClockwise);
            assert_eq!(mv.inverse().dir, mv.dir.inverse());
            assert_eq!(mv.dir.inverse().inverse(), mv.dir);
            assert_eq!(State::SOLVED.mv(mv).mv(mv.inverse()), State::SOLVED);
        }
    }
}
