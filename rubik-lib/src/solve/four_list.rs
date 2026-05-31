use std::collections::HashMap;

use itertools::Itertools;
use smallvec::{SmallVec, smallvec};

pub mod product;
pub mod trie;

use crate::{
    model::{
        moves::{Move, Moves},
        state::State,
    },
    solve::four_list::product::Product,
};

/// Solves a rubik's cube using the four-list algorithm.
/// Given a scrambled state `s`, it tries to find four move sequences
/// `s1, s2, s3, s4` such that `s4 * s3 * s2 * s1 = s`.
/// It does so by searching from both sides simultaneously until an
/// intersection is found in the middle:
/// `s2 * s1 = s3' * s4' * s`
/// It is possible to search in both sides by iterating in sorted order
/// and advancing the iterators side by side.
/// Using permutation tries, it is possible to iterate over cartesian
/// products of permutations in sorted order.
pub struct Solver {
    state_to_moves: HashMap<State, Moves>,
    left_iter: Product,
    right_iter: Product,
    left: Option<(State, State, State)>,
    right: Option<(State, State, State)>,
    scrambled: State,
}

impl Solver {
    pub fn new(scrambled: State, quarter_depth: u8) -> Solver {
        let state_to_moves: HashMap<_, _> = Moves::to_depth(quarter_depth)
            .unique_by(|&(_, s)| s)
            .map(|(m, s)| (s, m))
            .collect();
        let mut left_iter = Product::sorted(
            state_to_moves.keys().cloned(),
            state_to_moves.keys().cloned(),
        );
        let mut right_iter = Product::sorted(
            state_to_moves.keys().map(|s| s.inv()),
            state_to_moves.keys().map(|s| s.inv() * scrambled),
        );
        let mut left = None;
        let mut right = None;
        Self::advance(&mut left_iter, &mut left);
        Self::advance(&mut right_iter, &mut right);
        Self {
            state_to_moves,
            left_iter,
            right_iter,
            left,
            right,
            scrambled,
        }
    }

    pub fn exhausted(&self) -> bool {
        self.left.is_none() || self.right.is_none()
    }

    pub fn step(&mut self) -> Option<SmallVec<[Move; 20]>> {
        let (s2, s1, s2s1) = self.left?;
        let (s3, s4s, s3s4s) = self.right?;
        if s2s1 < s3s4s {
            Self::advance(&mut self.left_iter, &mut self.left);
            None
        } else if s2s1 > s3s4s {
            Self::advance(&mut self.right_iter, &mut self.right);
            None
        } else {
            assert_eq!(s2s1, s3s4s);
            let mut moves = smallvec![];
            moves.extend(self.state_to_moves[&s1].moves());
            moves.extend(self.state_to_moves[&s2].moves());
            moves.extend(self.state_to_moves[&(s3.inv())].moves());
            moves.extend(self.state_to_moves[&(s4s * self.scrambled.inv()).inv()].moves());
            Some(moves)
        }
    }

    fn advance(iter: &mut Product, res: &mut Option<(State, State, State)>) {
        if let Some(abc) = iter.next() {
            res.replace(abc);
        }
    }
}

impl Iterator for Solver {
    type Item = SmallVec<[Move; 20]>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.exhausted() {
            let step = self.step();
            if step.is_some() {
                return step;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        model::{moves::Move, state::State},
        solve::four_list::Solver,
    };

    fn scramble(n: usize) -> (Vec<Move>, State) {
        let mut i = 0;
        let mut moves = Vec::with_capacity(n);
        let mut state = State::ID;
        for _ in 0..n {
            let mv = Move::BASIC_MOVES[i];
            moves.push(mv);
            state = state.mv(mv);
            i = (i + 7) % 18;
        }
        (moves, state)
    }

    #[test]
    fn solve8() {
        let (moves, scrambled) = scramble(8);
        let mut solver = Solver::new(scrambled, 2);
        let solution = solver.next();
        assert!(solution.is_some());
        let solution = solution.unwrap();
        assert_eq!(solution.iter().copied().collect::<Vec<_>>(), moves);
        let state = solution
            .into_iter()
            .fold(State::ID, |state, mv| state.mv(mv));
        assert_eq!(state, scrambled);
    }
}
