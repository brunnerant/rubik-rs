use rubik_lib::{trie::Trie, util::Moves};

fn main() {
    let mut trie = Trie::new();
    let mut count = 0;
    for (_, state) in Moves::to_depth(5).iter() {
        count += 1;
        trie.insert(state);
    }
    println!("number of states: {}", count);
    println!("branching factor: {:.2}", trie.branching());
    println!("memory use:       {} MB", trie.footprint() / 1024 / 1024);
}
