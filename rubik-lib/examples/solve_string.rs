use std::{
    io::stdin,
    sync::mpsc::{Receiver, Sender, channel},
};

use itertools::join;
use rubik_lib::{
    core::{moves::Move, state::State},
    solve::kociemba::{self, Until},
};
use smallvec::SmallVec;

fn next_chars_from_user() -> String {
    let mut chars = String::new();
    for line in stdin().lines() {
        let line = line.unwrap();
        chars.extend(line.chars().filter(|c| !c.is_whitespace()));
        if chars.len() >= 54 {
            break;
        }
    }
    chars
}

fn next_cube_from_user() -> State {
    loop {
        if let Some(state) = State::from_string(&next_chars_from_user()) {
            return state;
        }
        println!("invalid input");
    }
}

fn print_sol(sol: &SmallVec<[u8; 30]>) {
    println!(
        "{} moves: {}",
        sol.len(),
        join(sol.iter().map(|&i| Move::BASIC_MOVES[i as usize]), " ")
    );
}

pub struct StdInListener {
    start_to_listen: Sender<()>,
    enter_pressed: Receiver<()>,
    listening: bool,
}

impl StdInListener {
    pub fn new() -> Self {
        let (start_to_listen, thread_start_to_listen) = channel();
        let (thread_enter_pressed, enter_pressed) = channel();
        std::thread::spawn(move || {
            while thread_start_to_listen.recv().is_ok() {
                stdin().lines().next();
                let _ = thread_enter_pressed.send(());
            }
        });
        Self {
            start_to_listen,
            enter_pressed,
            listening: false,
        }
    }

    pub fn enter_pressed(&mut self) -> bool {
        if !self.listening {
            self.listening = true;
            let _ = self.start_to_listen.send(());
        }
        let pressed = self.enter_pressed.try_recv().is_ok();
        if pressed {
            self.listening = false;
        }
        pressed
    }
}

fn main() {
    let mut solver = kociemba::Solver::from_folder("data/kociemba").expect("failed to init solver");
    let mut listener = StdInListener::new();
    loop {
        let state = next_cube_from_user();
        solver.init(&state);
        println!("searching for solutions...");
        println!("press <Enter> to stop searching.");
        print_sol(&solver.next().unwrap());
        loop {
            match solver.step_until(Until::MaxIterations(10000)) {
                Some(None) => {
                    println!("exhausted solution space. last solution is optimal.");
                    break;
                }
                Some(Some(sol)) => {
                    print_sol(&sol);
                }
                None => {}
            }
            if listener.enter_pressed() {
                println!("stopped searching for solutions.");
                break;
            }
        }
    }
}
