pub mod coords;
pub mod pruning;

use std::{
    path::Path,
    time::{Duration, Instant},
};

pub use coords::Tables;
use smallvec::{SmallVec, smallvec};

use crate::{
    algebra::coord::{CP, Coord},
    core::state::State,
    solve::kociemba::coords::{Coords, PHASE2_MOVES, Phase1, Phase2, is_phase2_move},
};

#[derive(Clone, Copy)]
pub enum Until {
    NextSolution,
    MaxIterations(usize),
}

impl Iterator for Until {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Until::NextSolution => Some(()),
            Until::MaxIterations(0) => None,
            Until::MaxIterations(n) => {
                *n -= 1;
                Some(())
            }
        }
    }
}

#[derive(Default, Clone, Copy)]
struct Record<C: Coords> {
    coords: C,
    dist: u8,
    next_mv: u8,
}

pub struct Solver {
    tables: Tables,

    init: State,
    best: SmallVec<[u8; 30]>,

    phase1: SmallVec<[Record<Phase1>; 12]>,
    phase1_init: Record<Phase1>,
    phase1_target_len: u8,
    phase1_zero_done: bool,

    phase2: SmallVec<[Record<Phase2>; 18]>,
    phase2_init: Record<Phase2>,
    phase2_target_len: u8,
    phase2_max_len: u8,

    in_phase2: bool,
}

impl Solver {
    pub fn from_folder(folder: impl AsRef<Path>) -> std::io::Result<Self> {
        let folder = folder.as_ref();
        let tables = Tables::from_folder(folder)?;
        Ok(Self {
            tables,

            init: State::ID,
            best: smallvec![0; 30],

            phase1: smallvec![],
            phase1_init: Default::default(),
            phase1_target_len: 0,
            phase1_zero_done: false,

            phase2: smallvec![],
            phase2_init: Default::default(),
            phase2_target_len: 0,
            phase2_max_len: 0,

            in_phase2: false,
        })
    }

    pub fn init(&mut self, state: &State) {
        self.phase1.clear();
        self.in_phase2 = false;
        self.best = smallvec![0; 30];
        self.init = *state;
        self.phase1_init.coords = Coords::from_state(state, &self.tables);
        self.phase1_init.dist = phase1_min_len(self.phase1_init.coords, &self.tables);
        self.phase1_target_len = self.phase1_init.dist;
        self.phase1_zero_done = self.phase1_target_len > 0;
    }

    fn phase1_step(&mut self) -> Option<bool> {
        // Retrieve the last move to perform
        let Some(last) = self.phase1.last_mut() else {
            // Special case when the initial state is already in the target space
            if !self.phase1_zero_done && self.phase1_init.coords.reached_goal() {
                // Keep the phase1 array empty, but move on in the next iteration
                self.phase1_zero_done = true;
                return Some(true);
            }

            // If there is no such move, move on to the next depth (IDA),
            // except if we reached the maximal depth of 20
            if self.phase1_target_len + 1 >= self.best.len().min(21) as u8 {
                return Some(false);
            }
            self.phase1_target_len += 1;
            self.phase1.push(self.phase1_init);
            return None;
        };
        let Record {
            coords,
            dist,
            next_mv: mv,
        } = *last;
        last.next_mv += 1;

        // If we haven't exhausted the moves yet, consider the next move
        if mv < 18 {
            // Prune the moves by disallowing repeated moves on the same / opposite face
            if self.phase1.len() > 1 {
                let prev_mv = self.phase1[self.phase1.len() - 2].next_mv - 1;
                if prune_move(prev_mv, mv) {
                    return None;
                }
            }

            // Compute the new distance
            let next_coords = coords.mv(mv, &self.tables);
            let dist_mod_3 = next_coords.min_dist(&self.tables);
            let new_dist = update_dist(dist, dist_mod_3);
            let rem_steps = self.phase1_target_len - self.phase1.len() as u8;

            // Prune it if it cannot reach the goal in the target number of moves
            if new_dist > rem_steps {
                return None;
            }

            // Return the solution if we reached the target number of moves
            if self.phase1.len() as u8 == self.phase1_target_len {
                // Don't allow to end phase1 with a phase2 move, as we can
                // perform the move in phase2
                if is_phase2_move(mv) {
                    return None;
                } else {
                    return Some(true);
                }
            }

            // Go to the next move
            self.phase1.push(Record {
                dist: new_dist,
                next_mv: 0,
                coords: next_coords,
            });
        } else {
            // If the moves are exhausted, go back to the parent
            self.phase1.pop();
        }
        None
    }

    fn phase2_step(&mut self) -> Option<bool> {
        // Retrieve the last move to perform
        let Some(last) = self.phase2.last_mut() else {
            // Special case for empty phase 2
            if self.phase2_target_len == 0 && self.phase2_init.coords.reached_goal() {
                return Some(true);
            }

            // If there is no such move, move on to the next depth (IDA),
            // except if we reached the max depth
            if self.phase2_target_len == self.phase2_max_len {
                return Some(false);
            }
            self.phase2_target_len += 1;
            self.phase2.push(self.phase2_init);
            return None;
        };
        let Record {
            coords,
            dist,
            next_mv: mv,
        } = *last;
        last.next_mv += 1;

        // If we haven't exhausted the moves yet, consider the next move
        if mv < PHASE2_MOVES.len() as u8 {
            let mv = PHASE2_MOVES[mv as usize];

            // Prune the moves by disallowing repeated moves on the same / opposite face
            if self.phase2.len() > 1 {
                let prev_mv_idx = self.phase2[self.phase2.len() - 2].next_mv - 1;
                let prev_mv = PHASE2_MOVES[prev_mv_idx as usize];
                if prune_move(prev_mv, mv) {
                    return None;
                }
            }

            // Compute the new distance
            let next_coords = coords.mv(mv, &self.tables);
            let dist_mod_3 = next_coords.min_dist(&self.tables);
            let new_dist = update_dist(dist, dist_mod_3);
            let rem_steps = self.phase2_target_len - self.phase2.len() as u8;

            // Prune it if it cannot reach the goal in the target number of moves
            if new_dist > rem_steps {
                return None;
            }

            // Return the solution if we reached the target number of moves
            if self.phase2.len() as u8 == self.phase2_target_len {
                // The other coordinates are zero, but we must check the ep4 coord
                if next_coords.ep4 == 0 {
                    return Some(true);
                } else {
                    return None;
                }
            }

            // Go to the next move
            self.phase2.push(Record {
                dist: new_dist,
                coords: next_coords,
                next_mv: 0,
            });
        } else {
            // If the moves are exhausted, go back to the parent
            self.phase2.pop();
        }
        None
    }

    fn init_phase2(&mut self) {
        let mut state = self.init;
        for s in self.phase1.iter() {
            let mv = s.next_mv - 1;
            state = state * State::BASIC_MOVES[mv as usize];
        }

        self.phase2_init.coords = Coords::from_state(&state, &self.tables);
        self.phase2_init.next_mv = 0;
        self.phase2_init.dist = phase2_min_len(self.phase2_init.coords, &self.tables);
        self.phase2_max_len = (self.best.len() - self.phase1.len() - 1) as u8;
        self.phase2_target_len = self.phase2_init.dist.min(self.phase2_max_len);
        self.phase2.clear();
    }

    pub fn step_until(&mut self, until: Until) -> Option<Option<SmallVec<[u8; 30]>>> {
        for _ in until {
            if self.in_phase2 {
                match self.phase2_step() {
                    None => {}
                    Some(true) => {
                        let mut moves = SmallVec::<[u8; 30]>::new();
                        moves.extend(self.phase1.iter().map(|s| s.next_mv - 1));
                        moves.extend(
                            self.phase2
                                .iter()
                                .map(|s| PHASE2_MOVES[s.next_mv as usize - 1]),
                        );
                        assert!(moves.len() <= self.best.len());
                        self.best = moves.clone();
                        if self.phase1_target_len >= self.best.len() as u8 {
                            self.phase1.clear();
                        }
                        self.in_phase2 = false;
                        return Some(Some(moves));
                    }
                    Some(false) => {
                        self.in_phase2 = false;
                    }
                }
            } else {
                match self.phase1_step() {
                    None => {}
                    Some(true) => {
                        self.init_phase2();
                        self.in_phase2 = true;
                    }
                    Some(false) => return Some(None),
                }
            }
        }
        None
    }

    pub fn solve_timeout(&mut self, state: &State, timeout: Duration) -> SmallVec<[u8; 30]> {
        // Find the first solution, and then try to improve it within the given timeout
        self.init(state);
        let timeout = Instant::now() + timeout;
        let mut best_sol = self.next().unwrap();

        loop {
            match self.step_until(Until::MaxIterations(1000)) {
                None => {}           // next solution not yet reached
                Some(None) => break, // solutions exhausted
                Some(Some(mvs)) => {
                    best_sol = mvs;
                }
            }
            if Instant::now() >= timeout {
                break;
            }
        }

        best_sol
    }
}

impl Iterator for Solver {
    type Item = SmallVec<[u8; 30]>;

    fn next(&mut self) -> Option<SmallVec<[u8; 30]>> {
        self.step_until(Until::NextSolution).unwrap()
    }
}

fn phase1_min_len(mut coords: Phase1, tables: &Tables) -> u8 {
    let mut num_moves = 0;
    let mut next_d = (coords.min_dist(tables) + 2) % 3;
    while !coords.reached_goal() {
        coords = (0..18)
            .find_map(|i| {
                let next_coords = coords.mv(i, tables);
                (next_coords.min_dist(tables) == next_d).then_some(next_coords)
            })
            .expect("invalid pruning table: no move was found that decreases the distance");
        num_moves += 1;
        next_d = (next_d + 2) % 3;
    }
    num_moves
}

fn phase2_min_len(mut coords: Phase2, tables: &Tables) -> u8 {
    let mut num_moves = 0;
    let mut next_d = (coords.min_dist(tables) + 2) % 3;
    while CP::unpack_sym_coord(coords.cp).0 != 0 || coords.ep8 != 0 {
        coords = PHASE2_MOVES
            .iter()
            .find_map(|&i| {
                let next_coords = coords.mv(i, tables);
                (next_coords.min_dist(tables) == next_d).then_some(next_coords)
            })
            .expect("invalid pruning table: no move was found that decreases the distance");
        num_moves += 1;
        next_d = (next_d + 2) % 3;
    }
    num_moves
}

fn prune_move(prev_mv: u8, mv: u8) -> bool {
    let prev_face = prev_mv / 3;
    let face = mv / 3;
    let prev_axis = prev_mv / 6;
    let axis = mv / 6;
    prev_axis == axis && (prev_face + 1) != face
}

fn update_dist(prev_dist: u8, new_dist_mod_3: u8) -> u8 {
    if new_dist_mod_3 == (prev_dist + 2) % 3 {
        prev_dist - 1
    } else if new_dist_mod_3 == (prev_dist + 1) % 3 {
        prev_dist + 1
    } else {
        prev_dist
    }
}
