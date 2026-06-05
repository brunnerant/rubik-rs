use std::io::Write;

use itertools::join;
use rubik_lib::{
    core::{moves::Move, scramble::scramble},
    solve::four_list::Solver,
};

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
