pub struct State {
    // Each corner contains its position (3 bits) and orientation (2 bits), compared to the solved cube
    pub corners: bitvec::BitArr!(for 8 * (3 + 2), in u64),
    // Each edge contains its position (4 bits) and orientation (1 bit), compared to the solved cube
    pub edges: bitvec::BitArr!(for 12 * (4 + 1), in u64),
}
