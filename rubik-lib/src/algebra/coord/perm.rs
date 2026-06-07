use std::collections::HashSet;

use crate::{algebra::coord::Coord, core::state::State};

/// Corner orientation coordinate
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
        let mut coord = 0;
        let mut left_set: u8 = 0;
        for i in 0..7 {
            let cpi = state.cp(i);
            let num_smaller = (left_set & ((1 << cpi) - 1)).count_ones();
            coord *= 8 - i as u16;
            coord += cpi as u16 - num_smaller as u16;
            left_set |= 1 << cpi;
        }
        Self { coord }
    }

    fn from_repr(repr: Self::Raw) -> Self {
        Self { coord: repr }
    }

    fn repr(&self) -> Self::Raw {
        self.coord
    }

    fn sample_state(&self) -> State {
        let perm = perm(self.coord);

        let mut corners = 0;
        for (i, p) in perm.into_iter().enumerate() {
            corners |= (p as u64) << (5 * i + 2);
        }

        let a = (self.coord / 24) % 2 == 0;
        let b = (self.coord.div_ceil(2)) % 2 == 0;
        let c = (self.coord / 720) % 2 == 0;
        let even = a ^ b ^ c;
        let edges = if even {
            State::ID.edges
        } else {
            0b_10110_10100_10010_10000_01110_01100_01010_01000_00110_00100_00000_00010
        };

        State { corners, edges }
    }
}

fn perm(mut n: u16) -> [u8; 8] {
    let mut digits = [0; 7];
    for i in 0..7 {
        let k = i + 2;
        digits[6 - i as usize] = n % k;
        n /= k;
    }
    let mut taken = HashSet::new();
    let mut perm = [0; 8];
    for i in 0..7 {
        let mut digit = digits[i];
        let mut result = 0;
        loop {
            if !taken.contains(&result) {
                if digit == 0 {
                    break;
                }
                digit -= 1;
            }
            result += 1;
        }
        perm[i] = result;
        taken.insert(result);
    }

    assert_eq!(7, taken.len());
    for i in 0..8 {
        if !taken.contains(&i) {
            perm[7] = i;
            break;
        }
    }
    perm
}

#[cfg(test)]
mod tests {

    use crate::algebra::coord::perm::perm;

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
            assert_eq!(a ^ b ^ c, even(perm(i)), "{i}");
        }
    }
}
