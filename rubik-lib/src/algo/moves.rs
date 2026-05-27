use std::collections::VecDeque;

use smallvec::{SmallVec, smallvec};

use crate::{model::moves::Move, model::state::State};

/// An encoded sequence of basic moves.
/// It is encoded using a power series with move indices.
/// Due to the chosen size of the encoding, the sequences can
/// be of length at most 15.
pub struct Moves {
    encoded: u64,
}

impl Moves {
    pub fn from_encoding(encoded: u64) -> Self {
        Self { encoded }
    }

    const EMPTY: Moves = Self { encoded: 0 };

    pub fn from_move(mv: Move) -> Self {
        Self {
            encoded: mv.index() as u64 + 1,
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
            count <= 15,
            "cannot encode sequences with more than 15 moves"
        );
        result
    }

    pub fn append(&self, mv: Move) -> Self {
        Self {
            encoded: 19 * self.encoded + mv.index() as u64 + 1,
        }
    }

    pub fn moves(&self) -> SmallVec<[Move; 15]> {
        let mut result = smallvec![];
        let mut encoding = self.encoded;
        while encoding > 0 {
            result.push(Move::BASIC_MOVES[((encoding % 19) - 1) as usize]);
            encoding /= 19;
        }
        result.reverse();
        result
    }

    pub fn inv_moves(&self) -> SmallVec<[Move; 15]> {
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
            max: 19_u64.pow(d as u32),
        }
    }
}

pub struct MovesIter {
    next: VecDeque<(Moves, State)>,
    max: u64,
}

impl Iterator for MovesIter {
    type Item = (Moves, State);

    fn next(&mut self) -> Option<Self::Item> {
        self.next.pop_front().map(|(moves, state)| {
            if 19 * moves.encoded < self.max {
                for mv in Move::BASIC_MOVES {
                    self.next.push_back((moves.append(mv), state.mv(mv)));
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

    use crate::{
        algo::moves::Moves,
        model::{moves::Move, state::State},
    };

    #[test]
    fn unique_moves_to_depth_5() {
        assert_eq!(Moves::to_depth(5).unique_by(|&(_, s)| s).count(), 621649);
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
            .fold(State::ID, |state, mv| state.mv(mv));
        let inv_state = moves
            .inv_moves()
            .into_iter()
            .fold(State::ID, |state, mv| state.mv(mv));
        assert_eq!(state.inv(), inv_state);
    }
}
