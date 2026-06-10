pub mod coords;
pub mod pruning;

use std::path::Path;

pub use coords::Coords;
use smallvec::{SmallVec, smallvec};

use crate::{
    algebra::coord::{CO, CP, Coord, EOLR, EP4, EP8},
    core::{io::BinarySerde, state::State},
    solve::kociemba::pruning::PruningTable,
};

#[derive(Default, Clone, Copy)]
struct Phase1Record {
    dist: u8,
    next_mv: u8,
    eolr: u32,
    co: u16,
}

#[derive(Default, Clone, Copy)]
struct Phase2Record {
    dist: u8,
    next_mv: u8,
    cp: u16,
    ep8: u16,
    ep4: u8,
}

pub struct Solver {
    coords: Coords,
    phase1_pruning: PruningTable<EOLR, CO>,
    phase2_pruning: PruningTable<CP, EP8>,

    init: State,
    best: SmallVec<[u8; 30]>,

    phase1: SmallVec<[Phase1Record; 12]>,
    phase1_init: Phase1Record,
    phase1_target_len: u8,
    phase1_zero_done: bool,

    phase2: SmallVec<[Phase2Record; 18]>,
    phase2_init: Phase2Record,
    phase2_target_len: u8,
    phase2_max_len: u8,
}

impl Solver {
    pub fn from_folder(folder: impl AsRef<Path>) -> std::io::Result<Self> {
        let folder = folder.as_ref();
        let coords = Coords::from_folder(folder)?;
        let phase1_pruning = PruningTable::from_file(folder.join("phase1-pruning.bin"))?;
        let phase2_pruning = PruningTable::from_file(folder.join("phase2-pruning.bin"))?;
        Ok(Self {
            coords,
            phase1_pruning,
            phase2_pruning,

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
        })
    }

    pub fn init(&mut self, state: &State) {
        self.phase1.clear();
        self.best = smallvec![0; 30];
        self.init = *state;
        self.phase1_init.eolr = self
            .coords
            .eolr_coord
            .raw_to_sym(EOLR::from_state(state).coord());
        self.phase1_init.co = CO::from_state(state).coord();
        self.phase1_init.dist = phase1_min_len(
            self.phase1_init.eolr,
            self.phase1_init.co,
            &self.coords,
            &self.phase1_pruning,
        );
        self.phase1_target_len = self.phase1_init.dist;
        self.phase1_zero_done = self.phase1_target_len > 0;
    }

    fn phase1_step(&mut self) -> bool {
        loop {
            // Retrieve the last move to perform
            let Some(last) = self.phase1.last_mut() else {
                // Special case when the initial state is already in the target space
                if !self.phase1_zero_done
                    && EOLR::unpack_sym_coord(self.phase1_init.eolr).0 == 0
                    && self.phase1_init.co == 0
                {
                    // Keep the phase1 array empty, but move on in the next iteration
                    self.phase1_zero_done = true;
                    return true;
                }

                // If there is no such move, move on to the next depth (IDA),
                // except if we reached the maximal depth of 20
                if self.phase1_target_len >= self.best.len().min(20) as u8 {
                    return false;
                }
                self.phase1_target_len += 1;
                self.phase1.push(self.phase1_init);
                continue;
            };
            let mv = last.next_mv;
            let dist = last.dist;
            let eolr = last.eolr;
            let co = last.co;
            last.next_mv += 1;

            // If we haven't exhausted the moves yet, consider the next move
            if mv < 18 {
                // Prune the moves by disallowing repeated moves on the same / opposite face
                if self.phase1.len() > 1 {
                    let prev_mv = self.phase1[self.phase1.len() - 2].next_mv - 1;
                    if prune_move(prev_mv, mv) {
                        continue;
                    }
                }

                // Compute the new distance
                let eolr = self.coords.eolr_mv.coord_mv(eolr, mv, &self.coords.sym);
                let co = self.coords.co_mv.coord_mv(co, mv);
                let dist_mod_3 =
                    self.phase1_pruning
                        .dist(eolr, co, &self.coords.sym, &self.coords.co_sym);
                let new_dist = update_dist(dist, dist_mod_3);
                let rem_steps = self.phase1_target_len - self.phase1.len() as u8;

                // Prune it if it cannot reach the goal in the target number of moves
                if new_dist > rem_steps {
                    continue;
                }

                // Return the solution if we reached the target number of moves
                if self.phase1.len() as u8 == self.phase1_target_len {
                    // Don't allow to end phase1 with a phase2 move, as we can
                    // perform the move in phase2
                    if is_phase2_move(mv) {
                        continue;
                    } else {
                        return true;
                    }
                }

                // Go to the next move
                self.phase1.push(Phase1Record {
                    dist: new_dist,
                    next_mv: 0,
                    eolr,
                    co,
                });
            } else {
                // If the moves are exhausted, go back to the parent
                self.phase1.pop();
            }
        }
    }

    fn phase2_step(&mut self) -> bool {
        if self.phase2_target_len == 0
            && CP::unpack_sym_coord(self.phase2_init.cp).0 == 0
            && self.phase2_init.ep8 == 0
            && self.phase2_init.ep4 == 0
        {
            return true;
        }

        loop {
            // Retrieve the last move to perform
            let Some(last) = self.phase2.last_mut() else {
                // If there is no such move, move on to the next depth (IDA),
                // except if we reached the max depth
                if self.phase2_target_len == self.phase2_max_len {
                    return false;
                }
                self.phase2_target_len += 1;
                self.phase2.push(self.phase2_init);
                continue;
            };
            let mv = last.next_mv;
            let dist = last.dist;
            let cp = last.cp;
            let ep8 = last.ep8;
            let ep4 = last.ep4;
            last.next_mv += 1;

            // If we haven't exhausted the moves yet, consider the next move
            if mv < 18 {
                // Only consider phase 2 moves
                if !is_phase2_move(mv) {
                    continue;
                }

                // Prune the moves by disallowing repeated moves on the same / opposite face
                if self.phase2.len() > 1 {
                    let prev_mv = self.phase2[self.phase2.len() - 2].next_mv - 1;
                    if prune_move(prev_mv, mv) {
                        continue;
                    }
                }

                // Compute the new distance
                let cp = self.coords.cp_mv.coord_mv(cp, mv, &self.coords.sym);
                let ep8 = self.coords.ep8_mv.coord_mv(ep8, mv);
                let ep4 = self.coords.ep4_mv.coord_mv(ep4, mv);
                let dist_mod_3 =
                    self.phase2_pruning
                        .dist(cp, ep8, &self.coords.sym, &self.coords.ep8_sym);
                let new_dist = update_dist(dist, dist_mod_3);
                let rem_steps = self.phase2_target_len - self.phase2.len() as u8;

                // Prune it if it cannot reach the goal in the target number of moves
                if new_dist > rem_steps {
                    continue;
                }

                // Return the solution if we reached the target number of moves
                if self.phase2.len() as u8 == self.phase2_target_len {
                    // The other coordinates are zero, but we must check the ep4 coord
                    if ep4 != 0 {
                        continue;
                    } else {
                        return true;
                    }
                }

                // Go to the next move
                self.phase2.push(Phase2Record {
                    dist: new_dist,
                    next_mv: 0,
                    cp,
                    ep8,
                    ep4,
                });
            } else {
                // If the moves are exhausted, go back to the parent
                self.phase2.pop();
            }
        }
    }

    fn init_phase2(&mut self) {
        let mut state = self.init;
        for s in self.phase1.iter() {
            let mv = s.next_mv - 1;
            state = state * State::BASIC_MOVES[mv as usize];
        }

        self.phase2_init.cp = self
            .coords
            .cp_coord
            .raw_to_sym(CP::from_state(&state).coord());
        self.phase2_init.ep8 = EP8::from_state(&state).coord();
        self.phase2_init.ep4 = EP4::from_state(&state).coord();
        self.phase2_init.next_mv = 0;
        self.phase2_init.dist = phase2_min_len(
            self.phase2_init.cp,
            self.phase2_init.ep8,
            &self.coords,
            &self.phase2_pruning,
        );
        self.phase2_max_len = (self.best.len() - self.phase1.len() - 1) as u8;
        self.phase2_target_len = self.phase2_init.dist.min(self.phase2_max_len);
        self.phase2.clear();
    }

    pub fn step(&mut self) -> Option<SmallVec<[u8; 30]>> {
        while self.phase1_step() {
            self.init_phase2();
            if self.phase2_step() {
                let mut moves = SmallVec::<[u8; 30]>::new();
                moves.extend(self.phase1.iter().map(|s| s.next_mv - 1));
                moves.extend(self.phase2.iter().map(|s| s.next_mv - 1));
                assert!(moves.len() <= self.best.len());
                self.best = moves.clone();
                if self.phase1_target_len >= self.best.len() as u8 {
                    self.phase1.clear();
                }
                return Some(moves);
            }
        }
        None
    }
}

fn phase1_min_len(
    mut eolr: u32,
    mut co: u16,
    coords: &Coords,
    phase1_pruning: &PruningTable<EOLR, CO>,
) -> u8 {
    let mut num_moves = 0;
    let mut next_d = (phase1_pruning.dist(eolr, co, &coords.sym, &coords.co_sym) + 2) % 3;
    while EOLR::unpack_sym_coord(eolr).0 != 0 || co != 0 {
        let (next_eolr, next_co) = (0..18)
            .find_map(|i| {
                let next_eolr = coords.eolr_mv.coord_mv(eolr, i, &coords.sym);
                let next_co = coords.co_mv.coord_mv(co, i);
                (phase1_pruning.dist(next_eolr, next_co, &coords.sym, &coords.co_sym) == next_d)
                    .then_some((next_eolr, next_co))
            })
            .expect("invalid pruning table: no move was found that decreases the distance");
        num_moves += 1;
        eolr = next_eolr;
        co = next_co;
        next_d = (next_d + 2) % 3;
    }
    num_moves
}

fn phase2_min_len(
    mut cp: u16,
    mut ep8: u16,
    coords: &Coords,
    phase2_pruning: &PruningTable<CP, EP8>,
) -> u8 {
    let mut num_moves = 0;
    let mut next_d = (phase2_pruning.dist(cp, ep8, &coords.sym, &coords.ep8_sym) + 2) % 3;
    while CP::unpack_sym_coord(cp).0 != 0 || ep8 != 0 {
        let (next_cp, next_ep8) = PHASE2_MOVES
            .iter()
            .find_map(|&i| {
                let next_cp = coords.cp_mv.coord_mv(cp, i, &coords.sym);
                let next_ep8 = coords.ep8_mv.coord_mv(ep8, i);
                (phase2_pruning.dist(next_cp, next_ep8, &coords.sym, &coords.ep8_sym) == next_d)
                    .then_some((next_cp, next_ep8))
            })
            .expect("invalid pruning table: no move was found that decreases the distance");
        num_moves += 1;
        cp = next_cp;
        ep8 = next_ep8;
        next_d = (next_d + 2) % 3;
    }
    num_moves
}

const PHASE2_MOVES: [u8; 10] = [0, 1, 2, 3, 4, 5, 8, 11, 14, 17];

fn is_phase2_move(mv: u8) -> bool {
    !(mv / 3 >= 2 && mv % 3 < 2)
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

#[cfg(test)]
mod tests {
    use crate::{core::moves::Move, solve::kociemba::is_phase2_move};

    #[test]
    fn phase2_moves() {
        assert_eq!(
            vec![
                Move::L,
                Move::L_,
                Move::L2,
                Move::R,
                Move::R_,
                Move::R2,
                Move::D2,
                Move::U2,
                Move::B2,
                Move::F2
            ],
            (0..18)
                .filter(|&i| is_phase2_move(i))
                .map(|i| Move::BASIC_MOVES[i as usize])
                .collect::<Vec<_>>()
        )
    }
}
