use itertools::Itertools;

use crate::state::State;

pub struct TrieBuilder {
    branches: Vec<usize>,
    states: Vec<State>,
}

// branch structure:
// - 6 bits give the depth in the uncompressed trie
// - 4 bits give the index of the node with respect to its possible siblings
// - 4 bits for each possible child
//   - if 0, no child
//   - if >0, points to relative u64 that points to child
pub struct Trie {
    branches: Vec<u64>,
    states: Vec<State>,
}

impl Default for TrieBuilder {
    fn default() -> Self {
        TrieBuilder::new()
    }
}

impl TrieBuilder {
    const NO_CHILD: usize = usize::MAX;
    const MAX_DEPTH: u64 = 2 * 8 + 2 * 12;

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

    pub fn build(self) -> Trie {
        let mut branches = Vec::new();
        self.build_branch(0, 0, 0, &mut branches);
        Trie {
            branches,
            states: self.states,
        }
    }

    fn build_branch(
        &self,
        node: usize,
        d: u64,
        sibling_idx: u64,
        branches: &mut Vec<u64>,
    ) -> usize {
        let max_children = Self::max_children(d);
        let num_children = self.branches[node..node + max_children]
            .iter()
            .filter(|&&n| n != Self::NO_CHILD)
            .count();
        let header_offset = branches.len();
        branches.append(&mut vec![0; num_children + 1]);
        let mut child_idx = 0;
        let mut header = (d << 58) | (sibling_idx << 54);
        for (idx, &child) in self.branches[node..node + max_children]
            .iter()
            .enumerate()
            .filter(|&(_, &n)| n != Self::NO_CHILD)
        {
            let (mut child, child_d, sibling_idx) = self.segment_end(child, d + 1, idx as u64);
            if Self::is_leaf(child_d) {
                child |= 1 << 63;
            } else {
                child = self.build_branch(child, child_d, sibling_idx, branches);
            }

            child_idx += 1;
            header |= (child_idx as u64) << (4 * idx);
            branches[header_offset + child_idx] = child as u64;
        }
        branches[header_offset] = header;
        header_offset
    }

    fn segment_end(&self, mut node: usize, mut d: u64, mut sibling_idx: u64) -> (usize, u64, u64) {
        while !Self::is_leaf(d) {
            let max_children = Self::max_children(d);
            let Some((idx, &only_child)) = self.branches[node..node + max_children]
                .iter()
                .enumerate()
                .filter(|&(_, &n)| n != Self::NO_CHILD)
                .exactly_one()
                .ok()
            else {
                break;
            };
            node = only_child;
            d += 1;
            sibling_idx = idx as u64;
        }
        (node, d, sibling_idx)
    }

    fn is_leaf(s: u64) -> bool {
        s >= Self::MAX_DEPTH
    }

    fn max_children(s: u64) -> usize {
        match (s < 2 * 8, s.is_multiple_of(2)) {
            (true, true) => 8,
            (true, false) => 3,
            (false, true) => 12,
            (false, false) => 2,
        }
    }
}

impl Trie {
    pub fn footprint(&self) -> usize {
        self.branches.len() * size_of::<usize>() + self.states.len() * size_of::<State>()
    }
}

pub struct TriePtr {
    coset: State,
    stack: smallvec::SmallVec<[u64; 13]>,
}

impl TriePtr {
    pub fn first(coset: State) -> TriePtr {
        // This is similar to state inversion, except that the orientation part
        // is inverted but its position is not swapped.
        let mut corners = 0;
        for i in 0..8 {
            let pos = State::get_block(coset.corners, 5 * i + 2, 3) as u8;
            let ori = State::get_block(coset.corners, 5 * i, 2);
            corners |= (i as u64) << (5 * pos + 2);
            corners |= ((3 - ori) % 3) << (5 * i);
        }
        let mut edges = 0;
        for i in 0..12 {
            let pos = State::get_block(coset.edges, 5 * i + 1, 4) as u8;
            let ori = State::get_block(coset.edges, 5 * i, 1);
            edges |= (i as u64) << (5 * pos + 1);
            edges |= ori << (5 * i);
        }
        let coset = State { corners, edges };

        TriePtr {
            coset,
            stack: smallvec::smallvec![0],
        }
    }

    pub fn next(&mut self, trie: &Trie) -> Option<State> {
        while let Some(&top) = self.stack.last() {
            let node = State::get_block(top, 0, 56) as usize;
            let i = State::get_block(top, 60, 4) as u8;
            let branch = trie.branches[node];
            let d = State::get_block(branch, 58, 6);
            let n = TrieBuilder::max_children(d);

            if (i as usize) < n {
                // Go to the next branch
                *self.stack.last_mut().unwrap() += 1 << 60;
                let parent_i = State::get_block(branch, 54, 4) as u8;
                let j = match (d < 2 * 8, d.is_multiple_of(2)) {
                    (true, true) => State::get_block(self.coset.corners, 5 * i + 2, 3) as u8,
                    (true, false) => {
                        (i + State::get_block(self.coset.corners, 5 * parent_i, 2) as u8) % 3
                    }
                    (false, true) => State::get_block(self.coset.edges, 5 * i + 1, 4) as u8,
                    (false, false) => {
                        (i + State::get_block(self.coset.edges, 5 * parent_i, 1) as u8) % 2
                    }
                };

                // Push the child on the stack if it exists
                let child_idx = State::get_block(branch, 4 * j, 4);
                if child_idx != 0 {
                    let child = trie.branches[node + child_idx as usize];
                    let leaf = State::get_block(child, 0, 63);

                    // Return the state if the child is a leaf
                    if child != leaf {
                        return Some(trie.states[leaf as usize]);
                    }
                    self.stack.push(child | ((j as u64) << 56));
                }
            } else {
                // Pop the branch off the stack
                self.stack.pop();
            }
        }
        None
    }

    pub fn depth(&self) -> u8 {
        self.stack.len() as u8
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
    use crate::{moves::Move, state::State, trie::TrieBuilder, util::Moves};

    #[test]
    fn trie_normal_ordering() {
        let mut trie = TrieBuilder::new();
        let mut states: Vec<_> = Moves::to_depth(4).iter().map(|(_, s)| s).collect();
        for &state in states.iter() {
            trie.insert(state);
        }
        states.sort();
        let trie = trie.build();
        assert_eq!(states, trie.ordered().collect::<Vec<_>>());
    }

    #[test]
    fn trie_coset_ordering1() {
        let mut trie = TrieBuilder::new();
        let state1 = State::SOLVED;
        let state2 = state1.mv(Move::F);
        let state3 = state1.mv(Move::F_);
        trie.insert(state1);
        trie.insert(state2);
        trie.insert(state3);
        let trie = trie.build();
        let states: Vec<State> = trie.ordered_coset(State::SOLVED.mv(Move::F_)).collect();
        assert_eq!(states, vec![state2, state1, state3]);
    }

    #[test]
    fn trie_coset_ordering2() {
        let mut trie = TrieBuilder::new();
        let state1 = State::SOLVED.mv(Move::D_);
        let state2 = State::SOLVED.mv(Move::L);
        trie.insert(state1);
        trie.insert(state2);
        let trie = trie.build();
        let states: Vec<State> = trie.ordered_coset(State::SOLVED.mv(Move::F)).collect();
        assert_eq!(states, vec![state1, state2]);
    }

    #[test]
    fn trie_coset_ordering3() {
        let mut trie = TrieBuilder::new();
        let mut states: Vec<_> = Moves::to_depth(4).iter().map(|(_, s)| s).collect();
        for &state in states.iter() {
            trie.insert(state);
        }
        let coset = State::SOLVED.mv(Move::F2).mv(Move::U);
        for state in states.iter_mut() {
            *state = coset.compose(state);
        }
        let trie = trie.build();
        states.sort();
        assert_eq!(
            states,
            trie.ordered_coset(coset)
                .map(|s| coset.compose(&s))
                .collect::<Vec<_>>()
        );
    }
}
