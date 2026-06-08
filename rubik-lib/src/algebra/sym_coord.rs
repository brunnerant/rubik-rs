use std::collections::HashMap;

use itertools::Itertools;
use num::traits::FromBytes;
use smallvec::{SmallVec, smallvec};

use crate::{
    algebra::{coord::Coord, sym::Symmetries},
    core::{
        bits::{Int, deserialize_array, serialize_array},
        io::BinarySerde,
        state::State,
    },
};

/// Sym-coords are coordinates that are reduced by symmetries. This allows to have more
/// compact move tables and pruning tables.
/// For a given symmetry equivalence class, a single coordinate is used to represent the whole class.
/// To do so, element with the smallest coordinate from the equivalence class is taken as the representative
/// for its class. A mapping is then built from the raw coordinate of the representative to the
/// index of the equivalence class.
pub struct SymCoordTable<C: Coord> {
    // bijective mapping coord(Raw) = coord(Sj^-1 * Repr_i * Sj)
    raw_to_repr: Vec<C::Sym>, // retrieves i given Raw
    raw_to_sym: Vec<u8>,      // retrives j given Raw
    repr_to_raw: Vec<C::Raw>, // retrieves Raw given i (such that j = 0)

    // internal symmetries
    repr_to_symmetries: Vec<C::Sym>,
    symmetries: Vec<u8>,
}

impl<C: Coord> SymCoordTable<C> {
    pub fn build(sym: &Symmetries) -> Self {
        let mut raw_to_repr = vec![C::SYM_SIZE; C::raw_to_usize(C::RAW_SIZE)];
        let mut raw_to_sym = vec![0; C::raw_to_usize(C::RAW_SIZE)];
        let mut repr_to_raw = Vec::new();
        for c in C::all_raw_coords() {
            if raw_to_repr[C::raw_to_usize(c)] != C::SYM_SIZE {
                continue;
            }
            let s = C::from_repr(c).sample_state();
            let raw: Vec<_> = (0..sym.size())
                .map(|i| C::from_state(&sym.conj(s, i)).repr())
                .collect();
            let min = raw.iter().position_min().unwrap();
            let min_inv = sym.inv(min as u8);

            let repr_idx = C::usize_to_sym(repr_to_raw.len());
            for (i, &r) in raw.iter().enumerate() {
                let idx = C::raw_to_usize(r);
                raw_to_repr[idx] = repr_idx;
                raw_to_sym[idx] = sym.prod(min_inv, i as u8);
            }
            repr_to_raw.push(raw[min]);
        }
        assert_eq!(repr_to_raw.len(), C::sym_to_usize(C::SYM_SIZE));

        let mut symmetries = vec![];
        let mut repr_to_symmetries = Vec::with_capacity(C::raw_to_usize(C::RAW_SIZE));
        let mut syms_to_idx = HashMap::new();
        for &coord in repr_to_raw.iter() {
            let state = C::from_repr(coord).sample_state();
            let mut syms: SmallVec<[_; 16]> = smallvec![];
            for i in 0..sym.size() {
                if C::from_state(&sym.conj(state, i)).repr() == coord {
                    syms.push(i);
                }
            }
            if !syms_to_idx.contains_key(&syms) {
                syms_to_idx.insert(syms.clone(), symmetries.len());
                symmetries.push(syms.len() as u8);
                symmetries.extend(syms.iter().cloned());
            }
            repr_to_symmetries.push(C::usize_to_sym(syms_to_idx[&syms]));
        }
        Self {
            raw_to_repr,
            raw_to_sym,
            repr_to_raw,
            repr_to_symmetries,
            symmetries,
        }
    }

    pub fn internal_sym(&self, coord: C::Sym) -> &[u8] {
        let idx = C::sym_to_usize(self.repr_to_symmetries[C::sym_to_usize(coord)]);
        let num_sym = self.symmetries[idx] as usize;
        &self.symmetries[idx + 1..idx + num_sym + 1]
    }

    pub fn sym_coord(&self, state: State) -> C::Raw {
        let raw = C::raw_to_usize(C::from_state(&state).repr());
        C::pack_sym_coord(self.raw_to_repr[raw], self.raw_to_sym[raw])
    }

    pub fn raw_coord(&self, sym_coord: C::Raw, sym: &Symmetries) -> C::Raw {
        let (s, i) = C::unpack_sym_coord(sym_coord);
        let raw = self.repr_to_raw[C::sym_to_usize(s)];
        let s = sym.conj(C::from_repr(raw).sample_state(), i);
        C::from_state(&s).repr()
    }

    pub fn repr(&self, coord: C::Sym) -> State {
        let raw = self.repr_to_raw[C::sym_to_usize(coord)];
        C::from_repr(raw).sample_state()
    }

    pub fn size(&self) -> usize {
        self.repr_to_raw.len()
    }
}

impl<C: Coord> BinarySerde for SymCoordTable<C>
where
    for<'a> &'a [u8]: TryInto<&'a <C::Raw as FromBytes>::Bytes>,
    for<'a> &'a [u8]: TryInto<&'a <C::Sym as FromBytes>::Bytes>,
{
    fn to_binary(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&serialize_array(&self.raw_to_repr));
        buffer.extend_from_slice(&serialize_array(&self.raw_to_sym));
        buffer.extend_from_slice(&serialize_array(&self.repr_to_raw));
        buffer.extend_from_slice(&serialize_array(&self.repr_to_symmetries));
        buffer.extend_from_slice(&self.symmetries);
        buffer
    }

    fn from_binary(mut buffer: &[u8]) -> Option<Self> {
        let raw_to_repr = read_chunk(&mut buffer, C::raw_to_usize(C::RAW_SIZE));
        let raw_to_sym = read_chunk(&mut buffer, C::raw_to_usize(C::RAW_SIZE));
        let repr_to_raw = read_chunk(&mut buffer, C::sym_to_usize(C::SYM_SIZE));
        let repr_to_symmetries = read_chunk(&mut buffer, C::sym_to_usize(C::SYM_SIZE));
        Some(Self {
            raw_to_repr,
            raw_to_sym,
            repr_to_raw,
            repr_to_symmetries,
            symmetries: buffer.to_vec(),
        })
    }
}

fn read_chunk<T: Int>(buffer: &mut &[u8], len: usize) -> Vec<T>
where
    for<'a> &'a [u8]: TryInto<&'a <T as FromBytes>::Bytes>,
{
    let (head, tail) = buffer.split_at(len * size_of::<T>());
    *buffer = tail;
    deserialize_array(head)
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use crate::{
        algebra::{
            coord::{CO, CP, Coord, EOLR, LR},
            sym::Symmetries,
            sym_coord::SymCoordTable,
        },
        core::moves::Moves,
    };

    #[test]
    fn sym_coords() {
        fn test<C: Coord>(sym: &Symmetries) {
            let table = SymCoordTable::<C>::build(sym);
            for (_, s) in Moves::to_depth(3) {
                let raw_coord = C::from_state(&s).repr();
                let sym_coord = table.sym_coord(s);
                assert_eq!(raw_coord, table.raw_coord(sym_coord, sym));
            }
        }
        let sym = Symmetries::sub16();
        test::<CO>(&sym);
        // test::<EO>(&sym); The EO coord is not compatible with those symmetries.
        test::<LR>(&sym);
        test::<EOLR>(&sym);

        test::<CP>(&sym);
        // test::<EP8>(&sym);
        // test::<EP4>(&sym);
    }

    #[test]
    fn internal_sym() {
        fn test<C: Coord>(sym: &Symmetries) {
            let table = SymCoordTable::<C>::build(sym);
            for coord in C::all_sym_coords() {
                let raw = table.repr_to_raw[C::sym_to_usize(coord)];
                let state = C::from_repr(raw).sample_state();
                let internal: HashSet<_> = table.internal_sym(coord).iter().copied().collect();
                for s in 0..sym.size() {
                    let new_raw = C::from_state(&sym.conj(state, s)).repr();
                    assert_eq!(new_raw == raw, internal.contains(&s));
                }
            }
        }

        let sym = Symmetries::sub16();
        test::<CO>(&sym);
        test::<LR>(&sym);
        test::<EOLR>(&sym);

        test::<CP>(&sym);
        // test::<EP8>(&sym);
        // test::<EP4>(&sym);
    }
}
