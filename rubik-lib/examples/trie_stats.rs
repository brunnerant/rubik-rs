use itertools::Itertools;
use rubik_lib::{
    algo::{
        moves::Moves,
        trie::{TrieBuilder, TriePtr},
    },
    model::state::State,
};

fn main() {
    let mut trie = TrieBuilder::new();
    let mut count = 0;
    for (_, state) in Moves::to_depth(5).unique_by(|&(_, s)| s) {
        count += 1;
        trie.insert(state);
    }
    let trie = trie.build();

    let mut depth: usize = 0;
    let mut max_depth = 0;
    let mut count2: usize = 0;
    let mut ptr = TriePtr::first(State::ID);
    while ptr.next(&trie).is_some() {
        max_depth = max_depth.max(ptr.depth());
        depth += ptr.depth() as usize;
        count2 += 1;
    }
    let depth = depth as f64 / count2 as f64;
    assert_eq!(count, count2);

    println!("number of states: {}", count);
    println!("memory use:       {}MB", trie.footprint() / 1024 / 1024);
    println!("avg trie depth:   {:.1}", depth);
    println!("max trie depth:   {:.1}", max_depth);
    println!("size of ptr:      {}B", size_of_val(&ptr));
}
