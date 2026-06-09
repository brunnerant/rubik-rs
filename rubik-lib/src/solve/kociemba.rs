pub mod coords;
pub mod pruning;

use std::path::Path;

pub use coords::Coords;
use smallvec::{SmallVec, smallvec};

use crate::{
    algebra::coord::{CO, Coord, EOLR},
    core::{io::BinarySerde, moves::Move, state::State},
    solve::kociemba::pruning::PruningTable,
};

struct Phase1Record {
    dist: u8,
    next_mv: u8,
    eolr: u32,
    co: u16,
}

pub struct Solver {
    pub coords: Coords,
    phase1_pruning: PruningTable<EOLR, CO>,
    phase1_target_len: u8,
    phase1: SmallVec<[Phase1Record; 12]>,
    best: SmallVec<[u8; 31]>,
}

impl Solver {
    pub fn from_folder(folder: impl AsRef<Path>) -> std::io::Result<Self> {
        let folder = folder.as_ref();
        let coords = Coords::from_folder(folder)?;
        let phase1_pruning = PruningTable::from_file(folder.join("phase1-pruning.bin"))?;
        Ok(Self {
            coords,
            phase1_pruning,
            phase1_target_len: 0,
            phase1: smallvec![],
            best: smallvec![0; 31],
        })
    }

    pub fn init(&mut self, state: &State) {
        self.phase1.clear();
        self.best = smallvec![0; 31];
        let eolr = self
            .coords
            .eolr_coord
            .raw_to_sym(EOLR::from_state(state).coord());
        let co = CO::from_state(state).coord();
        self.phase1_target_len = phase1_min_len(eolr, co, &self.coords, &self.phase1_pruning);
        self.phase1.push(Phase1Record {
            dist: self.phase1_target_len,
            next_mv: 0,
            eolr,
            co,
        });
    }

    fn phase1_moves(&self) -> Vec<Move> {
        self.phase1
            .iter()
            .map(|s| {
                assert!(s.next_mv > 0);
                Move::BASIC_MOVES[s.next_mv as usize - 1]
            })
            .collect()
    }

    fn phase1_dist(&self, eolr: u32, co: u16, prev_dist: u8) -> u8 {
        let dist_mod_3 = self
            .phase1_pruning
            .dist(eolr, co, &self.coords.sym, &self.coords.co_sym);
        if dist_mod_3 == (prev_dist + 2) % 3 {
            prev_dist - 1
        } else if dist_mod_3 == (prev_dist + 1) % 3 {
            prev_dist + 1
        } else {
            prev_dist
        }
    }

    pub fn phase1_step(&mut self) -> Option<Vec<Move>> {
        if self.phase1_target_len == 0
            && let Some(prev) = self.phase1.last()
        {
            let (i, _) = EOLR::unpack_sym_coord(prev.eolr);
            if i == 0 && prev.co == 0 {
                self.phase1_target_len += 1;
                return Some(vec![]);
            }
        }
        while let Some(prev) = self.phase1.last_mut() {
            let mv = prev.next_mv;
            let dist = prev.dist;
            let eolr = prev.eolr;
            let co = prev.co;
            prev.next_mv += 1;
            if mv < 18 {
                if self.phase1.len() > 1 {
                    let prev_mv = self.phase1[self.phase1.len() - 2].next_mv - 1;
                    if prune_move(prev_mv, mv) {
                        continue;
                    }
                }

                let eolr = self.coords.eolr_mv.coord_mv(eolr, mv, &self.coords.sym);
                let co = self.coords.co_mv.coord_mv(co, mv);
                let new_dist = self.phase1_dist(eolr, co, dist);
                let rem_steps = self.phase1_target_len - self.phase1.len() as u8;
                if new_dist > rem_steps {
                    continue;
                }
                if self.phase1.len() as u8 == self.phase1_target_len {
                    if is_phase2_move(mv) {
                        continue;
                    } else {
                        return Some(self.phase1_moves());
                    }
                }
                self.phase1.push(Phase1Record {
                    dist: new_dist,
                    next_mv: 0,
                    eolr,
                    co,
                });
            } else {
                prev.next_mv = 0;
                if self.phase1.len() == 1 && self.phase1_target_len < 20 {
                    self.phase1_target_len += 1;
                } else {
                    self.phase1.pop();
                }
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
