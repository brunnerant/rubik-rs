use rubik_lib::{trie::Trie, util::Moves};

fn main() {
    let mut trie = Trie::new();
    for (_, state) in Moves::to_depth(5).iter() {
        trie.insert(state);
    }
    println!("{}", trie.branching());
}
