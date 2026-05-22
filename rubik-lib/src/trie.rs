use itertools::Itertools;

use crate::state::State;

pub struct Trie {
    branches: Vec<usize>,
    states: Vec<State>,
}

impl Default for Trie {
    fn default() -> Self {
        Trie::new()
    }
}

impl Trie {
    const NO_CHILD: usize = usize::MAX;

    pub fn new() -> Self {
        Self {
            branches: vec![Self::NO_CHILD; 8],
            states: vec![],
        }
    }

    pub fn insert(&mut self, state: State) {
        let mut branches = [(0, 0); 2 * 8 + 2 * 12];
        let mut b = 0;
        for i in 0..8 {
            let pos = State::get_block(state.corners, 5 * i + 2, 3);
            let ori = State::get_block(state.corners, 5 * i, 2);
            branches[b] = (pos as usize, 8);
            branches[b + 1] = (ori as usize, 3);
            b += 2;
        }
        for i in 0..12 {
            let pos = State::get_block(state.edges, 5 * i + 1, 4);
            let ori = State::get_block(state.edges, 5 * i, 1);
            branches[b] = (pos as usize, 12);
            branches[b + 1] = (ori as usize, 2);
            b += 2;
        }
        let mut node = 0;
        for (&(i, _), &(_, l)) in branches.iter().tuple_windows() {
            node = self.get_or_insert_branch(node, i, l);
        }
        self.insert_leaf(node, branches.last().unwrap().0, state);
    }

    fn get_or_insert_branch(&mut self, node: usize, pos: usize, size: usize) -> usize {
        if self.branches[node + pos] == Self::NO_CHILD {
            self.branches[node + pos] = self.branches.len();
            self.branches.append(&mut vec![Self::NO_CHILD; size]);
        }
        self.branches[node + pos]
    }

    fn insert_leaf(&mut self, node: usize, pos: usize, state: State) {
        self.branches[node + pos] = self.states.len();
        self.states.push(state);
    }

    fn is_leaf(s: usize) -> bool {
        s >= 2 * 8 + 2 * 12
    }

    fn num_children(s: usize) -> usize {
        match (s < 2 * 8, s % 2 == 0) {
            (true, true) => 8,
            (true, false) => 3,
            (false, true) => 12,
            (false, false) => 2,
        }
    }

    fn branching_impl(&self, node: usize, s: usize) -> (usize, usize) {
        if Self::is_leaf(s) {
            return (0, 0);
        }
        let num_children = Self::num_children(s);
        let num_branches = self.branches[node..node + num_children]
            .iter()
            .filter(|&&n| n != Self::NO_CHILD)
            .count();
        let (a, b) = self.branches[node..node + num_children]
            .iter()
            .filter(|&&n| n != Self::NO_CHILD)
            .map(|&n| self.branching_impl(n, s + 1))
            .fold((0, 0), |(s1, s2), (b1, b2)| (s1 + b1, s2 + b2));
        (num_branches + a, num_children + b)
    }

    pub fn branching(&self) -> f64 {
        let (a, b) = self.branching_impl(0, 0);
        if b == 0 { 0.0 } else { a as f64 / b as f64 }
    }
}

pub struct TriePtr {
    state: State,
    stack: [(usize, u8); 2 * 8 + 2 * 12],
    size: usize,
}

impl TriePtr {
    pub fn first(coset: State) -> TriePtr {
        TriePtr {
            state: coset.invert(),
            stack: [(0, 0); 2 * 8 + 2 * 12],
            size: 1,
        }
    }

    pub fn next(&mut self, trie: &Trie) -> Option<State> {
        while self.size > 0 {
            let s = self.size - 1;
            let (node, idx) = self.stack[s];
            let pos = idx >> 4; // the position of the corner or edge is given for orientation branches
            let i = idx & 0x0f;
            let n = Trie::num_children(s);

            if (i as usize) < n {
                // Go to the next branch
                self.stack[s].1 += 1;

                let j = match (s < 2 * 8, s % 2 == 0) {
                    (true, true) => State::get_block(self.state.corners, 5 * i + 2, 3) as u8,
                    (true, false) => {
                        (i + State::get_block(self.state.corners, 5 * pos, 2) as u8) % 3
                    }
                    (false, true) => State::get_block(self.state.edges, 5 * i + 1, 4) as u8,
                    (false, false) => {
                        (i + State::get_block(self.state.edges, 5 * pos, 1) as u8) % 2
                    }
                };

                let child_idx = trie.branches[node + j as usize];
                if child_idx != Trie::NO_CHILD {
                    if self.size >= self.stack.len() {
                        return Some(trie.states[child_idx]);
                    }
                    self.stack[self.size] = (child_idx, i << 4);
                    self.size += 1;
                }
            } else {
                // Pop the branch off the stack
                self.size -= 1;
            }
        }
        None
    }
}

pub struct TrieIter<'a> {
    trie: &'a Trie,
    ptr: TriePtr,
}

impl<'a> Iterator for TrieIter<'a> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        self.ptr.next(self.trie)
    }
}

impl Trie {
    pub fn ordered<'a>(&'a self) -> TrieIter<'a> {
        self.ordered_coset(State::SOLVED)
    }

    pub fn ordered_coset<'a>(&'a self, coset: State) -> TrieIter<'a> {
        TrieIter {
            trie: self,
            ptr: TriePtr::first(coset),
        }
    }
}

impl std::cmp::PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        fn rev(bits: u64, n: usize) -> u64 {
            let mut res = 0;
            for i in 0..n {
                res |= ((bits >> (5 * i)) & 0b11111) << (5 * (n - i - 1))
            }
            res
        }

        (rev(self.corners, 8), rev(self.edges, 12))
            .cmp(&(rev(other.corners, 8), rev(other.edges, 12)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{moves::Move, state::State, trie::Trie, util::Moves};

    #[test]
    fn trie_normal_ordering() {
        let mut trie = Trie::new();
        let mut states: Vec<_> = Moves::to_depth(4).iter().map(|(_, s)| s).collect();
        for &state in states.iter() {
            trie.insert(state);
        }
        states.sort();
        assert_eq!(states, trie.ordered().collect::<Vec<_>>());
    }

    #[test]
    fn trie_coset_ordering1() {
        let mut trie = Trie::new();
        let state1 = State::SOLVED;
        let state2 = state1.mv(Move::F);
        let state3 = state1.mv(Move::F_);
        trie.insert(state1);
        trie.insert(state2);
        trie.insert(state3);
        let states: Vec<State> = trie.ordered_coset(State::SOLVED.mv(Move::F_)).collect();
        assert_eq!(states, vec![state2, state1, state3]);
    }

    #[test]
    fn trie_coset_ordering2() {
        let mut trie = Trie::new();
        let state1 = State::SOLVED.mv(Move::D_);
        let state2 = State::SOLVED.mv(Move::L);
        trie.insert(state1);
        trie.insert(state2);
        let states: Vec<State> = trie.ordered_coset(State::SOLVED.mv(Move::F)).collect();
        assert_eq!(states, vec![state1, state2]);
    }

    #[test]
    fn trie_coset_ordering3() {
        let mut trie = Trie::new();
        let mut states: Vec<_> = Moves::to_depth(4).iter().map(|(_, s)| s).collect();
        for &state in states.iter() {
            trie.insert(state);
        }
        let coset = State::SOLVED.mv(Move::F2).mv(Move::U);
        for state in states.iter_mut() {
            *state = coset.compose(state);
        }
        states.sort();
        assert_eq!(
            states,
            trie.ordered_coset(coset)
                .map(|s| coset.compose(&s))
                .collect::<Vec<_>>()
        );
    }
}
