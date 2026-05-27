use std::collections::HashSet;

use crate::{model::moves::Move, model::state::State};

pub struct Moves {
    moves: Vec<(Move, usize, State)>,
}

impl Moves {
    pub fn to_depth(depth: usize) -> Self {
        if depth == 0 {
            return Self { moves: vec![] };
        }
        let mut moves = vec![];
        let mut states_to_check = vec![(State::SOLVED, usize::MAX)];
        let mut visited = HashSet::from([State::SOLVED]);
        for _ in 0..depth {
            let mut next_states_to_check = Vec::new();
            for (state, last_mv) in states_to_check.drain(..) {
                for mv in Move::BASIC_MOVES {
                    let next_state = state.mv(mv);
                    if visited.insert(next_state) {
                        next_states_to_check.push((next_state, moves.len()));
                        moves.push((mv, last_mv, next_state));
                    }
                }
            }
            states_to_check.append(&mut next_states_to_check);
        }
        Self { moves }
    }

    pub fn count(&self) -> usize {
        self.moves.len()
    }

    pub fn iter<'a>(&'a self) -> MoveIter<'a> {
        MoveIter {
            moves: &self.moves,
            idx: 0,
        }
    }
}

pub struct MoveIter<'a> {
    moves: &'a [(Move, usize, State)],
    idx: usize,
}

impl<'a> Iterator for MoveIter<'a> {
    type Item = (Vec<Move>, State);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.moves.len() {
            return None;
        }
        let mut result = Vec::new();
        let mut idx = self.idx;
        let state = self.moves[idx].2;
        loop {
            let (mv, last_idx, _) = self.moves[idx];
            result.push(mv);
            if last_idx == usize::MAX {
                break;
            }
            idx = last_idx;
        }
        self.idx += 1;
        result.reverse();
        Some((result, state))
    }
}

#[cfg(test)]
mod tests {
    use crate::algo::moves::Moves;

    #[test]
    fn unique_moves_to_depth_5() {
        assert_eq!(Moves::to_depth(5).count(), 621648);
    }
}
