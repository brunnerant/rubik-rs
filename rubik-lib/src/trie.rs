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
        for i in (0..8).rev() {
            let pos = State::get_block(state.corners, 5 * i + 2, 3);
            let ori = State::get_block(state.corners, 5 * i, 2);
            branches[b] = (pos as usize, 8);
            branches[b + 1] = (ori as usize, 3);
            b += 2;
        }
        for i in (0..12).rev() {
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
    stack: Vec<(&'a Trie, usize)>,
}

impl Iterator for TrieIter<'_> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, i)) = self.stack.pop() {
            match node {
                Trie::Branch { children } => {
                    if i < children.len() {
                        self.stack.push((node, i + 1));
                        if let Some(child) = &children[i] {
                            self.stack.push((child, 0));
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
    pub fn iter<'a>(&'a self) -> TrieIter<'a> {
        let mut node = self;
        let mut stack = Vec::new();
        loop {
            match node {
                Trie::Branch { children } => {
                    if let Some(i) = children.iter().position(Option::is_some) {
                        stack.push((node, i + 1));
                        node = children[i].as_ref().unwrap();
                    } else {
                        return TrieIter { stack: Vec::new() };
                    }
                }
                Trie::Leaf { .. } => {
                    stack.push((node, 0));
                    return TrieIter { stack };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{moves::Move, util::Moves};

use super::*;
    #[test]
    fn test_trie_insert_and_iter() {
        let mut trie = Trie::new();
        let state1 = State::SOLVED;
        let state2 = state1.mv(Move::F);
        let state3 = state1.mv(Move::F_);
        trie.insert(state1);
        trie.insert(state2);
        trie.insert(state3);
        let states: Vec<State> = trie.iter().collect();
        assert_eq!(states, vec![state3, state2, state1]);
    }

    #[test]
    fn test_trie_ordering() {
        let mut trie = Trie::new();
        let mut states: Vec<_> = Moves::to_depth(4).iter().map(|(_, s)| s).collect();
        states.sort_by_key(|s| (s.corners, s.edges));
        for &state in states.iter() {
            trie.insert(state);
        }
    }
}
