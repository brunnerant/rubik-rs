use rubik_lib::{trie::Trie, util::Moves};

fn main() {
    let mut trie = Trie::new();
    let mut count = 0;
    for (_, state) in Moves::to_depth(5).iter() {
        count += 1;
        trie.insert(state);
    }
    let stats = trie.stats();
    println!("number of states:   {}", count);
    println!("avg branching:      {:.2}", stats.avg_branching());
    println!("avg segment length: {:.1}", stats.avg_segment_length());
    println!("memory use:         {} MB", trie.footprint() / 1024 / 1024);
}
