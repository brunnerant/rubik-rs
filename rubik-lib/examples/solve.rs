use itertools::join;
use rand::RngExt;
use rubik_lib::{
    algo::four_list::Solver,
    model::{moves::Move, state::State},
};

fn scramble(n: usize) -> (Vec<Move>, State) {
    let mut rng = rand::rng();
    let mut moves = Vec::with_capacity(n);
    let mut state = State::ID;
    for _ in 0..n {
        let mv = Move::BASIC_MOVES[rng.random_range(0..18)];
        moves.push(mv);
        state = state.mv(mv);
    }
    (moves, state)
}

fn main() {
    let (_, scrambled) = scramble(100);
    let mut solver = Solver::new(scrambled, 5);
    let mut steps = 0;
    let solution = loop {
        if solver.exhausted() {
            break None;
        } else if let Some(sol) = solver.step() {
            break Some(sol);
        }
        steps += 1;
        let steps_m = steps / 1_000_000;
        print!("\rSteps: {}M", steps_m);
    };
    if let Some(solution) = solution {
        println!("Found solution:");
        println!("{}", join(solution.iter().map(|mv| mv.to_string()), " "));
    } else {
        println!("No solution found.");
    }
}
