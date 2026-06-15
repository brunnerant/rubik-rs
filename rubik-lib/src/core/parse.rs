use std::{collections::HashMap, ops::BitOr};

use itertools::Itertools;

use crate::core::{
    moves::{Axis, Face},
    state::State,
};

impl State {
    /// These utilities are useful to build a cube state out of its sticker positions.
    /// To specify a cube, write out the color of all its stickers as if it was projected on a sheet of paper.
    /// The order of the stickers should respect the following diagram, as read from left to right and top to bottom:
    /// ```raw
    /// UUU
    /// UUU
    /// UUU
    /// FFFRRRBBBLLL
    /// FFFRRRBBBLLL
    /// FFFRRRBBBLLL
    /// DDD
    /// DDD
    /// DDD
    /// ```
    pub fn from_stickers(stickers: &[Face]) -> Option<Self> {
        if stickers.len() != 54 {
            return None;
        }
        let stickers: Vec<_> = (0..54)
            .map(|i| {
                let idiv3 = (i / 3) % 3;
                let imod3 = i % 3;
                match i {
                    0..9 => stickers[9 + idiv3 * 12 + imod3 + 9],
                    9..18 => stickers[9 + idiv3 * 12 + imod3 + 3],
                    18..27 => stickers[i + 27],
                    27..36 => stickers[i - 27],
                    36..45 => stickers[9 + idiv3 * 12 + imod3 + 6],
                    45..54 => stickers[9 + idiv3 * 12 + imod3],
                    _ => unreachable!(),
                }
            })
            .collect();
        for i in 0..6 {
            if stickers[9 * i as usize + 4].index() != i {
                return None;
            }
        }
        const CORNER_IDX: [[usize; 3]; 8] = [
            [6, 6, 8],
            [8, 8, 6],
            [0, 0, 2],
            [2, 2, 0],
            [8, 0, 6],
            [6, 2, 8],
            [2, 6, 0],
            [0, 8, 2],
        ];
        let mut corners = 0;
        for i in 0..8 {
            let sides = [0, 1, 2].map(|j| {
                let f = 2 * j + ((i >> j) & 0b1);
                stickers[9 * f + CORNER_IDX[i][j]]
            });
            if !sides.iter().map(|f| f.axis()).all_unique() {
                return None;
            }
            let pos = sides
                .iter()
                .map(|f| (f.index() - 2 * f.axis().index()) << f.axis().index())
                .reduce(BitOr::bitor)
                .unwrap();
            let ori = sides.iter().position(|f| f.axis() == Axis::X).unwrap() as u8;
            let ori = if i.count_ones().is_multiple_of(2) {
                ori
            } else {
                (3 - ori) % 3
            };
            corners |= (pos as u64) << (5 * i + 2);
            corners |= (ori as u64) << (5 * i);
        }
        const EDGE_IDX: [[usize; 2]; 12] = [
            [7, 7],
            [1, 1],
            [1, 7],
            [7, 1],
            [3, 5],
            [5, 3],
            [5, 3],
            [3, 5],
            [7, 3],
            [7, 5],
            [1, 3],
            [1, 5],
        ];
        let mut edges = 0;
        for i in 0..12 {
            let [a, b] = (0..3)
                .filter(|&j| j != i / 4)
                .enumerate()
                .map(|(j, a)| {
                    let f = 2 * a + (((i % 4) >> j) & 0b1);
                    stickers[9 * f + EDGE_IDX[i][j]].index()
                })
                .collect_array()
                .unwrap();
            if a / 2 == b / 2 {
                return None;
            }
            let axis = (0..3).find(|&x| x != a / 2 && x != b / 2).unwrap();
            let (a1, a2) = if a < b { (a, b) } else { (b, a) };
            let pos = 4 * axis + (a1 % 2) + ((a2 % 2) << 1);
            edges |= (pos as u64) << (5 * i + 1);
            edges |= ((a > b) as u64) << (5 * i);
        }
        let state = State { corners, edges };
        state.valid().then_some(state)
    }

    pub fn from_string(string: &str) -> Option<Self> {
        let chars: [char; 54] = string
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect_array()?;
        let face_mapping = HashMap::from([
            (chars[4], Face::Up),
            (chars[22], Face::Front),
            (chars[25], Face::Right),
            (chars[28], Face::Back),
            (chars[31], Face::Left),
            (chars[49], Face::Down),
        ]);
        let mut stickers = Vec::with_capacity(54);
        for c in chars {
            stickers.push(*face_mapping.get(&c)?);
        }
        Self::from_stickers(&stickers)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::state::State;

    #[test]
    fn solved_from_string() {
        let cube = "\
            UUU\
            UUU\
            UUU\
            FFFRRRBBBLLL\
            FFFRRRBBBLLL\
            FFFRRRBBBLLL\
            DDD\
            DDD\
            DDD";
        assert_eq!(State::from_string(cube), Some(State::ID));
        let cube = "\
            111\
            111\
            111\
            222333444555\
            222333444555\
            222333444555\
            666\
            666\
            666";
        assert_eq!(State::from_string(cube), Some(State::ID));
    }

    #[test]
    fn face_turn_from_string() {
        let cube = "\
            BUU\
            BUU\
            BUU\
            UFFRRRBBDLLL\
            UFFRRRBBDLLL\
            UFFRRRBBDLLL\
            FDD\
            FDD\
            FDD";
        assert_eq!(State::from_string(cube), Some(State::L));
        let cube = "\
            UUU\
            UUU\
            UUU\
            RRRBBBLLLFFF\
            FFFRRRBBBLLL\
            FFFRRRBBBLLL\
            DDD\
            DDD\
            DDD";
        assert_eq!(State::from_string(cube), Some(State::U));
    }

    #[test]
    fn invalid_from_string() {
        let cube = "\
            UUU\
            UUU\
            UFU\
            FUFRRRBBBLLL\
            FFFRRRBBBLLL\
            FFFRRRBBBLLL\
            DDD\
            DDD\
            DDD";
        assert_eq!(State::from_string(cube), None);
        let cube = "\
            UUU\
            UUU\
            UFU\
            FUFRRRBBBLLL\
            FUFRRRBBBLLL\
            FFFRRRBBBLLL\
            DDD\
            DDD\
            DDD";
        assert_eq!(State::from_string(cube), None);
        let cube = "\
            UUU\
            UUU\
            FUU\
            FFFRRRBBBLLL\
            FFFRRRBBBLLL\
            FFFRRRBBBLLL\
            DDD\
            DDD\
            DDD";
        assert_eq!(State::from_string(cube), None);
        let cube = "\
            UUU\
            UUU\
            UFU\
            FFFRRRBBBLLL\
            FFFRRRBBBLLL\
            FFFRRRBBBLLL\
            DDD\
            DDD\
            DDD";
        assert_eq!(State::from_string(cube), None);
    }
}
