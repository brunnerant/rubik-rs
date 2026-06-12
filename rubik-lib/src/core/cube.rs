use crate::core::{moves::Face, state::State};

/// These utilities are useful to build a cube state out of its sticker positions.
/// This goes along with a format for specifying cubes:
/// - List the stickers of all faces, in the order L, F, R, B, U, D
/// - Within a face, list the stickers with the following order:
///     0 1 2
///     3 4 5
///     6 7 8
/// - The faces L, R, R, and B should be orientated by keeping the vertical axis up
/// - The up face should be oriented with the back up.
/// - The down face should be oriented with the front up.
///       U
///     L F R B
///       D
impl State {
    pub fn from_stickers(stickers: &[Face]) -> Option<Self> {
        if stickers.len() != 6 * 9 {
            return None;
        }

        todo!()
    }

    pub fn from_string(string: &str) -> Option<Self> {
        let mut stickers = Vec::with_capacity(string.len());
        for c in string.chars() {
            stickers.push(match c {
                'L' => Face::Left,
                'R' => Face::Right,
                'D' => Face::Down,
                'U' => Face::Up,
                'B' => Face::Back,
                'F' => Face::Front,
                _ => return None,
            });
        }
        Self::from_stickers(&stickers)
    }
}

type EdgePiece = (Face, Face);
type EdgePos = (Face, Face);
type CornerPiece = (Face, Face, Face);
type CornerPos = (Face, Face, Face);
