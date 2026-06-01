use std::collections::BinaryHeap;

use crate::{
    model::state::State,
    solve::four_list::trie::{Trie, TrieBuilder, TriePtr},
};

#[derive(Clone)]
struct Entry {
    left: State,
    right: State,
    right_iter: TriePtr,
}

#[derive(PartialEq, Eq, Clone)]
struct HeapElem {
    product: State,
    idx: usize,
}

impl std::cmp::Ord for HeapElem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.product.cmp(&other.product).reverse()
    }
}

impl std::cmp::PartialOrd for HeapElem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Entry {
    fn first(left: State, trie: &Trie) -> Option<Self> {
        let mut right_iter = TriePtr::first(left);
        right_iter.next(trie).map(|right| Self {
            left,
            right,
            right_iter,
        })
    }
}

#[derive(Clone)]
pub struct Product {
    trie: Trie,
    entries: Vec<Entry>,
    queue: BinaryHeap<HeapElem>,
}

/// Allows to iterate over the cartesian product of states in sorted order.
impl Product {
    pub fn sorted(left: impl Iterator<Item = State>, right: impl Iterator<Item = State>) -> Self {
        let mut trie = TrieBuilder::new();
        for s in right {
            trie.insert(s);
        }
        let trie = trie.build();
        let mut entries = Vec::new();
        let mut queue = BinaryHeap::new();
        for s in left {
            if let Some(entry) = Entry::first(s, &trie) {
                queue.push(HeapElem {
                    product: entry.left * entry.right,
                    idx: entries.len(),
                });
                entries.push(entry);
            }
        }
        Self {
            trie,
            entries,
            queue,
        }
    }
}

impl Iterator for Product {
    type Item = (State, State, State);

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop().map(|mut elem| {
            let entry = &mut self.entries[elem.idx];
            let left = entry.left;
            let right = entry.right;
            let product = elem.product;
            if let Some(state) = entry.right_iter.next(&self.trie) {
                entry.right = state;
                elem.product = entry.left * entry.right;
                self.queue.push(elem);
            };
            (left, right, product)
        })
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{
        model::{moves::Moves, state::State},
        solve::four_list::product::Product,
    };

    fn states_to_depth(d: u8) -> Vec<State> {
        Moves::to_depth(d).map(|(_, s)| s).unique().collect()
    }

    #[test]
    fn self_product() {
        let h = 2;
        let half: Vec<_> = states_to_depth(h);
        let mut sorted_full: Vec<_> = states_to_depth(2 * h);
        sorted_full.sort();

        let half_squared: Vec<_> = Product::sorted(half.iter().cloned(), half.iter().cloned())
            .map(|(_, _, p)| p)
            .unique()
            .collect();
        assert_eq!(half_squared, sorted_full);
    }
}
