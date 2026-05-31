//! A symmetry is a movement of the cube as a whole.
//! Some cube states are equivalent to one another with respect to some symmetry.
//! Intuitively, A is equivalent to B if it is possible to perform it with a change
//! of perspective (i.e. a symmetry), with the same result.
//! Or, in mathematical terms, A is equivalent to B if there is a symmetry S such that
//! `B = S^-1 * A * S`
//! The set of symmetries for a cube contains 48 unique symmetries that can be generated
//! by four elements.

use crate::model::state::State;

/// Rotating the whole cube clockwise around the LBD corner.
const ROT_LBD: State = State {
    corners: 0b_11100_01100_11000_01000_10100_00100_10000_00000,
    edges: 0b_01110_01010_01100_01000_00110_00010_00100_00000_10110_10100_10010_10000,
};

/// Rotating the whole cube clockwise around the L axis (same direction as the face turn L).
const ROT_L: State = State {
    corners: 0b_01100_01000_11100_11000_00100_00000_10100_10000,
    edges: 0b_01010_01000_01110_01100_10110_10100_10010_10000_00010_00110_00000_00100,
};

/// Mirroring the whole cube across the origin.
const ROT_U2: State = State {
    corners: 0b_01000_01100_00000_00100_11000_11100_10000_10100,
    edges: 0b_10100_10110_10000_10010_01000_01010_01100_01110_00010_00000_00110_00100,
};

/// Mirroring the whole cube across the LR axis.
const MIR_LR: State = State {
    corners: 0b_11000_11100_10000_10100_01000_01100_00000_00100,
    edges: 0b_10100_10110_10000_10010_01100_01110_01000_01010_00110_00100_00010_00000,
};

pub struct Symmetries {
    pub elems: Vec<State>,
}

impl Symmetries {
    pub fn all() -> Self {
        let mut elems = Vec::with_capacity(48);
        let mut state0 = State::ID;
        for _ in 0..3 {
            let mut state1 = state0;
            for _ in 0..4 {
                let mut state2 = state1;
                for _ in 0..2 {
                    let mut state3 = state2;
                    for _ in 0..2 {
                        elems.push(state3);
                        state3 = MIR_LR * state3;
                    }
                    state2 = ROT_U2 * state2;
                }
                state1 = ROT_L * state1;
            }
            state0 = ROT_LBD * state0;
        }
        Self { elems }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use itertools::iproduct;

    use crate::model::sym::Symmetries;

    #[test]
    fn all_symmetries_form_a_group() {
        let sym = Symmetries::all();
        let all_syms: HashSet<_> = sym.elems.iter().cloned().collect();
        assert_eq!(all_syms.len(), sym.elems.len());
        for (i, j) in iproduct!(0..48, 0..48) {
            assert!(
                all_syms.contains(&(sym.elems[i] * sym.elems[j])),
                "{} = {:?}{} = {:?}",
                i,
                sym.elems[i],
                j,
                sym.elems[j]
            );
            assert!(all_syms.contains(&(sym.elems[j] * sym.elems[i])));
            assert!(all_syms.contains(&sym.elems[i].inv()))
        }
    }
}
