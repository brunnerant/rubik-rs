use itertools::Itertools;

use crate::state::State;

#[derive(Clone)]
pub enum Trie {
    Branch { children: Vec<Option<Box<Trie>>> },
    Leaf { state: State },
}

impl Default for Trie {
    fn default() -> Self {
        Trie::new()
    }
}

impl Trie {
    pub fn new() -> Self {
        Trie::Branch {
            children: vec![None; 8],
        }
    }

    pub fn insert(&mut self, state: State) {
        let mut node = self;
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
        for (&(i, _), &(_, l)) in branches.iter().tuple_windows() {
            node = node.get_or_insert_branch(i, l);
        }
        node.insert_leaf(branches.last().unwrap().0, state);
    }

    fn get_or_insert_branch(&mut self, pos: usize, size: usize) -> &mut Trie {
        let Trie::Branch { children } = self else {
            panic!("expected branch");
        };
        if children[pos].is_none() {
            children[pos] = Some(Box::new(Trie::Branch {
                children: vec![None; size],
            }));
        }
        children[pos].as_mut().unwrap()
    }

    fn insert_leaf(&mut self, pos: usize, state: State) {
        let Trie::Branch { children } = self else {
            panic!("expected branch");
        };
        children[pos] = Some(Box::new(Trie::Leaf { state }));
    }
}

pub struct TrieIter<'a> {
    state: State,
    stack: Vec<(&'a Trie, u8)>,
}

impl Iterator for TrieIter<'_> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, i)) = self.stack.pop() {
            match node {
                Trie::Branch { children } => {
                    let child_idx = i & 0x0f;
                    let parent_idx = i >> 4;
                    if (child_idx as usize) < children.len() {
                        let s = self.stack.len();
                        self.stack.push((node, i + 1));
                        #[allow(clippy::collapsible_else_if)]
                        let j = if s < 2 * 8 {
                            if s % 2 == 0 {
                                State::get_block(self.state.corners, 5 * child_idx + 2, 3) as u8
                            } else {
                                (i + State::get_block(self.state.corners, 5 * parent_idx, 2) as u8)
                                    % 3
                            }
                        } else {
                            if s % 2 == 0 {
                                State::get_block(self.state.edges, 5 * child_idx + 1, 4) as u8
                            } else {
                                (i + State::get_block(self.state.edges, 5 * parent_idx, 1) as u8)
                                    % 2
                            }
                        };

                        if let Some(child) = &children[j as usize] {
                            let idx = if s % 2 == 0 { 0 } else { j << 4 };
                            self.stack.push((child, idx));
                        }
                    }
                }
                Trie::Leaf { state } => return Some(*state),
            }
        }
        None
    }
}

impl Trie {
    pub fn ordered<'a>(&'a self) -> TrieIter<'a> {
        self.ordered_coset(State::SOLVED)
    }

    pub fn ordered_coset<'a>(&'a self, coset: State) -> TrieIter<'a> {
        TrieIter {
            state: coset.invert(),
            stack: vec![(self, 0)],
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
    fn test_trie_ordering() {
        let mut trie = Trie::new();
        let mut states: Vec<_> = Moves::to_depth(3).iter().map(|(_, s)| s).collect();
        for &state in states.iter() {
            trie.insert(state);
        }
        states.sort();
        assert_eq!(states, trie.ordered().collect::<Vec<_>>());
    }

    #[test]
    fn test_trie_coset_ordering1() {
        let mut trie = Trie::new();
        let state1 = State::SOLVED;
        let state2 = state1.mv(Move::F);
        let state3 = state1.mv(Move::F_);
        trie.insert(state1);
        trie.insert(state2);
        trie.insert(state3);
        let states: Vec<State> = trie.ordered_coset(State::SOLVED.mv(Move::B)).collect();
        assert_eq!(states, vec![state2, state1, state3]);
    }

    fn test_trie_coset_ordering() {
        let mut trie = Trie::new();
        let mut states: Vec<_> = Moves::to_depth(2).iter().map(|(_, s)| s).collect();
        for &state in states.iter() {
            trie.insert(state);
        }
        let coset = State::SOLVED.mv(Move::F);
        states.sort_by_key(|s| coset.compose(s));
        assert_eq!(states, trie.ordered_coset(coset).collect::<Vec<_>>());
    }
}
