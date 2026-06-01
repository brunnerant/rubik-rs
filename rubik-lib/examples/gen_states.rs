use rubik_lib::model::{bits, state::State};

fn bits_to_string(bits: u64, n: u8) -> String {
    let mut s = String::from("0b");
    for i in (0..n).rev() {
        s.push_str(&format!("_{:05b}", bits::get(bits, 5 * i, 5)));
    }
    s
}

fn state_to_string(state: &State) -> String {
    format!(
        "State {{\n    corners: {},\n    edges: {},\n}}",
        bits_to_string(state.corners, 8),
        bits_to_string(state.edges, 12)
    )
}

fn state_from_perm(cp: [u8; 8], co: [u8; 8], ep: [u8; 12], eo: [u8; 12]) -> State {
    let mut corners = 0;
    for i in 0..8 {
        corners |= (cp[i] as u64) << (5 * i + 2);
        corners |= (co[i] as u64) << (5 * i);
    }
    let mut edges = 0;
    for i in 0..12 {
        edges |= (ep[i] as u64) << (5 * i + 1);
        edges |= (eo[i] as u64) << (5 * i);
    }
    State { corners, edges }
}

macro_rules! gen_states {
    ($id:ident) => {
        let name = stringify!($id);
        let state = $id;
        println!("const {}: State = {};", name, state_to_string(&state));
        println!(
            "const {}_: State = {};",
            name,
            state_to_string(&state.inv())
        );
        println!(
            "const {}2: State = {};",
            name,
            state_to_string(&(state * state))
        );
    };
}

const L: State = State {
    corners: 0b_11100_10000_10100_00000_01100_11000_00100_01000,
    edges: 0b_10110_01100_10010_01000_01110_10000_01010_10100_00110_00100_00010_00000,
};

const R: State = State {
    corners: 0b_01100_11000_11100_10000_00100_01000_10100_00000,
    edges: 0b_01010_10100_01110_10000_10110_01100_10010_01000_00110_00100_00010_00000,
};

const D: State = State {
    corners: 0b_11100_11000_00101_10110_01100_01000_00010_10001,
    edges: 0b_10110_10100_00001_00101_01110_01100_01010_01000_00110_10011_00010_10001,
};

const U: State = State {
    corners: 0b_11010_01001_10100_10000_11101_01110_00100_00000,
    edges: 0b_00111_00011_10010_10000_01110_01100_01010_01000_10101_00100_10111_00000,
};

const B: State = State {
    corners: 0b_11100_11000_10100_10000_01010_00001_01101_00110,
    edges: 0b_10110_10100_10010_10000_01110_01100_00010_00000_00110_00100_01000_01010,
};

const F: State = State {
    corners: 0b_10101_11110_10010_11001_01100_01000_00100_00000,
    edges: 0b_10110_10100_10010_10000_00100_00110_01010_01000_01110_01100_00010_00000,
};

fn main() {
    let cp = [0, 1, 2, 3, 4, 5, 6, 7];
    let co = [0, 0, 0, 0, 0, 0, 0, 0];
    let ep = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
    let eo = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let state = state_from_perm(cp, co, ep, eo);
    println!("const ID: State = {};", state_to_string(&state));

    gen_states!(L);
    gen_states!(R);
    gen_states!(D);
    gen_states!(U);
    gen_states!(B);
    gen_states!(F);
}
