use std::collections::BinaryHeap;

use crate::{
    algo::trie::{Trie, TrieBuilder, TriePtr},
    model::state::State,
};

#[derive(PartialEq, Eq)]
struct Entry {
    left_iter: TriePtr,
    left: State,
    right: State,
    product: State,
}

impl Entry {
    fn first(trie: &Trie, right: State) -> Option<Self> {
        let mut left_iter = TriePtr::first(right);
        left_iter.next(trie).map(|left| Self {
            left_iter,
            left,
            right,
            product: left * right,
        })
    }
}

impl std::cmp::Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.product.cmp(&other.product).reverse()
    }
}

impl std::cmp::PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Product {
    trie: Trie,
    queue: BinaryHeap<Entry>,
}

/// Allows to iterate over the cartesian product of states in sorted order.
impl Product {
    pub fn sorted(left: impl Iterator<Item = State>, right: impl Iterator<Item = State>) -> Self {
        let mut trie = TrieBuilder::new();
        for s in left {
            trie.insert(s);
        }
        let trie = trie.build();
        let mut queue = BinaryHeap::new();
        for s in right {
            if let Some(entry) = Entry::first(&trie, s) {
                queue.push(entry);
            }
        }
        Self { trie, queue }
    }
}

impl Iterator for Product {
    type Item = (State, State);

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop().map(|mut entry| {
            let left = entry.left;
            let right = entry.right;
            if let Some(state) = entry.left_iter.next(&self.trie) {
                entry.left = state;
                entry.product = entry.left * entry.right;
                self.queue.push(entry);
            };
            (left, right)
        })
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{
        algo::{moves::Moves, product::Product},
        model::state::State,
    };

    fn states_to_depth(d: u8) -> Vec<State> {
        Moves::to_depth(d)
            .unique_by(|&(_, s)| s)
            .map(|(_, s)| s)
            .collect()
    }

    #[test]
    fn self_product() {
        let h = 2;
        let half: Vec<_> = states_to_depth(h);
        let mut sorted_full: Vec<_> = states_to_depth(2 * h);
        sorted_full.sort();

        let half_squared: Vec<_> = Product::sorted(half.iter().cloned(), half.iter().cloned())
            .map(|(l, r)| l * r)
            .unique()
            .collect();
        assert_eq!(half_squared, sorted_full);
    }
}
