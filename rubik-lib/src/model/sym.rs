//! A symmetry is a movement of the cube as a whole.
//! Some cube states are equivalent to one another with respect to some symmetry.
//! Intuitively, A is equivalent to B if it is possible to perform it with a change
//! of perspective (i.e. a symmetry), with the same result.
//! Or, in mathematical terms, A is equivalent to B if there is a symmetry S such that
//! `B = S^-1 * A * S`
//! The set of symmetries for a cube contains 48 unique symmetries that can be generated
//! by four elements.

use crate::model::state::State;

/// Rotating the whole cube clockwise around the L axis (same direction as the face turn L).
pub const ROT_L: State = State {
    corners: 0b_01100_01000_11100_11000_00100_00000_10100_10000,
    edges: 0b_01010_01000_01110_01100_10110_10100_10010_10000_00011_00111_00001_00101,
};

/// Rotating the whole cube clockwise around the LBD corner.
pub const ROT_LBD: State = State {
    corners: 0b_11110_01101_11001_01010_10101_00110_10010_00001,
    edges: 0b_01111_01011_01101_01001_00111_00011_00101_00001_10110_10100_10010_10000,
};

/// Mirroring the whole cube across the origin.
pub const ROT_U2: State = State {
    corners: 0b_01000_01100_00000_00100_11000_11100_10000_10100,
    edges: 0b_10100_10110_10000_10010_01000_01010_01100_01110_00010_00000_00110_00100,
};

/// Mirroring the whole cube across the LR axis.
pub const MIR_LR: State = State {
    corners: 0b_11000_11100_10000_10100_01000_01100_00000_00100 | (1 << 63),
    edges: 0b_10100_10110_10000_10010_01100_01110_01000_01010_00110_00100_00010_00000,
};

pub struct Symmetries {
    pub elems: Vec<State>,
    pub elems_inv: Vec<State>,
}

impl Symmetries {
    /// All 48 symmetries of the cube group. They form a group and are all unique.
    pub fn all() -> Self {
        let mut elems = Vec::with_capacity(48);
        let mut elems_inv = Vec::with_capacity(48);
        let mut state = State::ID;
        for _ in 0..4 {
            for _ in 0..3 {
                for _ in 0..2 {
                    for _ in 0..2 {
                        elems.push(state);
                        elems_inv.push(state.inv());
                        state = state * MIR_LR;
                    }
                    state = state * ROT_U2;
                }
                state = state * ROT_LBD;
            }
            state = state * ROT_L;
        }
        Self { elems, elems_inv }
    }

    /// The state resulting from applying this state under the given symmetry index.
    pub fn conj(&self, state: State, sym_idx: u8) -> State {
        self.elems_inv[sym_idx as usize] * state * self.elems[sym_idx as usize]
    }

    /// The representative state for the sym-class of the given state.
    /// It returns an equivalent state, up to a symmetry. The index of the symmetry is also given.
    /// The representative is chosen as the smallest state that belongs to the equivalence class.
    pub fn repr(&self, state: State) -> (State, u8) {
        (0..self.elems.len())
            .map(|i| (self.elems[i] * state * self.elems_inv[i], i as u8))
            .min_by_key(|(s, _)| *s)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use itertools::{iproduct, join};

    use crate::model::{
        moves::{Direction, Face, Move},
        state::State,
        sym::{MIR_LR, ROT_L, ROT_LBD, ROT_U2, Symmetries},
    };

    fn test_conj(mv1: Move, mv2: Move, sym: State) {
        assert_eq!(sym.inv() * mv2 * sym, mv1.into());
    }

    fn test_order(mut state: State, order: usize) {
        let mut power = State::ID;
        for _ in 0..order {
            power = power * state;
        }
        assert_eq!(power, State::ID);
    }

    #[test]
    fn rot_lbd() {
        test_conj(Move::U, Move::F, ROT_LBD);
        test_conj(Move::F, Move::R, ROT_LBD);
        test_conj(Move::R, Move::U, ROT_LBD);
        test_conj(Move::L, Move::D, ROT_LBD);
        test_conj(Move::B, Move::L, ROT_LBD);
        test_conj(Move::D, Move::B, ROT_LBD);
        test_order(ROT_LBD, 3);
    }

    #[test]
    fn rot_l() {
        test_conj(Move::L, Move::L, ROT_L);
        test_conj(Move::R, Move::R, ROT_L);
        test_conj(Move::U, Move::F, ROT_L);
        test_conj(Move::F, Move::D, ROT_L);
        test_conj(Move::D, Move::B, ROT_L);
        test_conj(Move::B, Move::U, ROT_L);
        test_order(ROT_L, 4);
    }

    #[test]
    fn rot_u2() {
        test_conj(Move::U, Move::U, ROT_U2);
        test_conj(Move::D, Move::D, ROT_U2);
        test_conj(Move::L, Move::R, ROT_U2);
        test_conj(Move::R, Move::L, ROT_U2);
        test_conj(Move::B, Move::F, ROT_U2);
        test_conj(Move::F, Move::B, ROT_U2);
        test_order(ROT_U2, 2);
    }

    #[test]
    fn mir_lr() {
        test_conj(Move::L, Move::R_, MIR_LR);
        test_conj(Move::R, Move::L_, MIR_LR);
        test_conj(Move::D, Move::D_, MIR_LR);
        test_conj(Move::U, Move::U_, MIR_LR);
        test_conj(Move::B, Move::B_, MIR_LR);
        test_conj(Move::F, Move::F_, MIR_LR);
        test_order(MIR_LR, 2);
    }

    #[test]
    fn rot_lbd_l() {
        let sym = ROT_LBD * ROT_L.inv(); // equivalent to ROT_F'
        test_conj(Move::L, Move::D, sym);
        test_conj(Move::D, Move::R, sym);
        test_conj(Move::R, Move::U, sym);
        test_conj(Move::U, Move::L, sym);
        test_conj(Move::F, Move::F, sym);
        test_conj(Move::B, Move::B, sym);
    }

    #[test]
    fn all_symmetries_form_a_group() {
        let sym = Symmetries::all();
        assert_eq!(sym.elems.len(), 48);
        let all_syms: HashSet<_> = sym.elems.iter().cloned().collect();
        for (i, j) in iproduct!(0..48, 0..48) {
            assert!(all_syms.contains(&(sym.elems[i] * sym.elems[j])), "{} {}", i, j);
            assert!(all_syms.contains(&sym.elems[i].inv()), "{}", i);
        }
    }

    #[test]
    fn state_repr() {
        let sym = Symmetries::all();
        let mut reprs = HashMap::new();
        for dir in [
            Direction::Clockwise,
            Direction::CounterClockwise,
            Direction::HalfTurn,
        ] {
            for face in Face::ALL_FACES {
                let mv = Move { face, dir };
                let state = State::ID.mv(mv);
                let (repr, sidx) = sym.repr(state);
                // assert_eq!(state, sym.conj(repr, sidx));
                assert!(repr.valid());
                reprs.entry(repr).or_insert(Vec::new()).push(mv);
            }
        }
        for v in reprs.values() {
            println!("{}", join(v.iter().map(Move::to_string), " "));
        }
        assert_eq!(
            reprs.values().cloned().collect::<HashSet<_>>(),
            HashSet::from([
                vec![Move::L2, Move::R2, Move::D2, Move::U2, Move::B2, Move::F2],
                vec![
                    Move::L,
                    Move::R,
                    Move::D,
                    Move::U,
                    Move::B,
                    Move::F,
                    Move::L_,
                    Move::R_,
                    Move::D_,
                    Move::U_,
                    Move::B_,
                    Move::F_
                ]
            ])
        );
    }
}
