use crate::core::{moves::Move, state::State};

use rand::RngExt;

pub fn scramble(n: usize) -> (Vec<Move>, State) {
    let mut rng = rand::rng();
    let mut moves: Vec<Move> = Vec::with_capacity(n);
    let mut state = State::ID;
    for _ in 0..n {
        let mut possible_moves: Vec<_> = Move::BASIC_MOVES.to_vec();
        if let Some(l1) = moves.last() {
            possible_moves.retain(|mv| mv.face != l1.face);
        }
        if let Some([l1, l2]) = moves.last_chunk()
            && l1.face.axis() == l2.face.axis()
        {
            possible_moves.retain(|mv| mv.face.axis() != l1.face.axis());
        }
        let mv = possible_moves[rng.random_range(0..possible_moves.len())];
        moves.push(mv);
        state = mv * state;
    }
    (moves, state)
}
