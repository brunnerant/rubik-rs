use crate::{
    algebra::coord::Coord,
    core::{
        bits::{self},
        state::State,
    },
};

/// Corner permutation coordinate
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CP {
    coord: u16,
}

impl Coord for CP {
    type Raw = u16;
    type Sym = u16;
    const RAW_SIZE: Self::Raw = 40320; // 8!
    const SYM_SIZE: Self::Sym = 40320;

    fn from_state(state: &State) -> Self {
        Self {
            coord: perm_to_coord::<8>(state.corners >> 2, 3),
        }
    }

    fn from_repr(repr: Self::Raw) -> Self {
        Self { coord: repr }
    }

    fn repr(&self) -> Self::Raw {
        self.coord
    }

    fn sample_state(&self) -> State {
        let (mut corners, even) = coord_to_perm::<8>(self.coord);
        corners <<= 2;
        let edges = if even {
            State::ID.edges
        } else {
            0b_10110_10100_10010_10000_01110_01100_01010_01000_00110_00100_00000_00010
        };
        State { corners, edges }
    }
}

/// Edge permutation coordinate for the L and R faces
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EP8 {
    coord: u16,
}

impl Coord for EP8 {
    type Raw = u16;
    type Sym = u16;
    const RAW_SIZE: Self::Raw = 40320; // 8!
    const SYM_SIZE: Self::Sym = 40320;

    fn from_state(state: &State) -> Self {
        let edges = (state.edges >> 21) - 0b_00100_00100_00100_00100_00100_00100_00100_00100;
        Self {
            coord: perm_to_coord::<8>(edges, 4),
        }
    }

    fn from_repr(repr: Self::Raw) -> Self {
        Self { coord: repr }
    }

    fn repr(&self) -> Self::Raw {
        self.coord
    }

    fn sample_state(&self) -> State {
        let (mut edges, even) = coord_to_perm::<8>(self.coord);
        edges += 0b_00100_00100_00100_00100_00100_00100_00100_00100;
        edges <<= 21;
        if even {
            edges |= 0b_00110_00100_00010_00000;
        } else {
            edges |= 0b_00110_00100_00000_00010;
        };
        State {
            corners: State::ID.corners,
            edges,
        }
    }
}

fn perm_to_coord<const N: usize>(perm: u64, k: u8) -> u16 {
    let mut coord = 0;
    let mut left_set: u8 = 0;
    for i in 0..N as u8 - 1 {
        let cpi = bits::get(perm, 5 * i, k);
        let num_smaller = (left_set & ((1 << cpi) - 1)).count_ones();
        coord *= N as u16 - i as u16;
        coord += cpi as u16 - num_smaller as u16;
        left_set |= 1 << cpi;
    }
    coord
}

fn coord_to_perm<const N: usize>(mut coord: u16) -> (u64, bool) {
    let mut digits = [0; N];
    for i in 0..N as u16 - 1 {
        let k = i + 2;
        digits[6 - i as usize] = coord % k;
        coord /= k;
    }
    let mut avail = [0, 1, 2, 3, 4, 5, 6, 7];
    let mut corners = 0;
    let mut even = true;
    for (i, d) in digits.into_iter().enumerate() {
        let pos = d as usize;
        corners |= avail[pos] << (5 * i);
        for j in pos..N - 1 - i {
            avail[j] = avail[j + 1];
        }
        even ^= !(N - 1 - i - pos).is_multiple_of(2);
    }
    (corners | (avail[0] << 35), even)
}

#[cfg(test)]
mod tests {
    fn perm(mut n: u16) -> ([u8; 8], bool) {
        let mut digits = [0; 7];
        for i in 0..7 {
            let k = i + 2;
            digits[6 - i as usize] = n % k;
            n /= k;
        }

        let mut avail = [0, 1, 2, 3, 4, 5, 6, 7];
        let mut perm = [0; 8];
        let mut even = true;
        for i in 0..7 {
            let pos = digits[i] as usize;
            perm[i] = avail[pos];
            for j in pos..7 - i {
                avail[j] = avail[j + 1];
            }
            even ^= !(7 - i - pos).is_multiple_of(2);
        }
        perm[7] = avail[0];
        (perm, even)
    }

    fn even(mut perm: [u8; 8]) -> bool {
        let mut even = true;
        for i in 0..8 {
            while perm[i] != i as u8 {
                let j = perm[i];
                perm.swap(i, j as usize);
                even = !even;
            }
        }
        even
    }

    #[test]
    fn parity() {
        for i in 0..40320 {
            let a = (i / 24) % 2 == 0;
            let b = ((i + 1) / 2) % 2 == 0;
            let c = (i / 720) % 2 == 0;
            let (p, e) = perm(i);
            assert_eq!(a ^ b ^ c, even(p), "{i}");
            assert_eq!(even(p), e);
        }
    }
}
