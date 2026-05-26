use rubik_lib::{trie::TrieBuilder, util::Moves};

fn main() {
    let mut trie = TrieBuilder::new();
    let mut count = 0;
    for (_, state) in Moves::to_depth(5).iter() {
        count += 1;
        trie.insert(state);
    }
    let trie = trie.build();
    println!("number of states: {}", count);
    println!("memory use:       {} MB", trie.footprint() / 1024 / 1024);
}
