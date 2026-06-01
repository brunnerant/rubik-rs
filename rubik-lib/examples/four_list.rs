use std::io::Write;

use itertools::join;
use rand::RngExt;
use rubik_lib::{
    model::{moves::Move, state::State},
    solve::four_list::Solver,
};

fn scramble(n: usize) -> (Vec<Move>, State) {
    let mut rng = rand::rng();
    let mut moves: Vec<Move> = Vec::with_capacity(n);
    let mut state = State::ID;
    for _ in 0..n {
        let mut possible_moves: Vec<_> = Move::BASIC_MOVES.iter().cloned().collect();
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

fn main() {
    let (moves, scrambled) = scramble(100);
    println!("Scrambled the cube:");
    println!("{}", join(moves.iter().map(Move::to_string), " "));
    let mut solver = Solver::new(scrambled, 5);
    let mut steps: usize = 0;
    let solution = loop {
        if solver.exhausted() {
            break None;
        } else if let Some(sol) = solver.step() {
            break Some(sol);
        }
        steps += 1;
        if steps.is_multiple_of(1_000_000) {
            let steps_m = steps / 1_000_000;
            print!("\rSearching... ({}M states searched)", steps_m);
            std::io::stdout().flush().unwrap();
        }
    };
    println!();
    if let Some(solution) = solution {
        println!("Found solution:");
        println!(
            "{}",
            join(solution.iter().rev().map(|mv| mv.inv().to_string()), " ")
        );
    } else {
        println!("No solution found.");
    }
}
