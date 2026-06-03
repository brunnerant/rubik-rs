use std::{
    collections::HashMap,
    io::{Read, Write},
};

use num::{Zero, traits::FromBytes};

use crate::{
    algebra::{coord::Coord, sym::Symmetries},
    core::{
        bits::{self, serialize_array},
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
    raw_to_repr: HashMap<C::Raw, C::Sym>,
    pub repr_to_raw: Vec<C::Raw>,
}

impl<C: Coord> SymCoordTable<C> {
    pub fn build(sym: &Symmetries) -> Self {
        let mut raw_to_repr = HashMap::new();
        for c in C::all_raw_coords() {
            let s = C::from_repr(c).sample_state();
            let raw = (0..sym.size())
                .map(|i| C::from_state(&sym.conj_inv(s, i)).repr())
                .min()
                .unwrap();
            if !raw_to_repr.contains_key(&raw) {
                raw_to_repr.insert(
                    raw,
                    num::cast::<usize, C::Sym>(raw_to_repr.len())
                        .expect("sym-coord type too small to hold the coordinates"),
                );
            }
        }
        assert_eq!(raw_to_repr.len(), C::sym_to_usize(C::SYM_SIZE));
        let mut repr_to_raw = vec![Zero::zero(); raw_to_repr.len()];
        for (&raw, &sym) in raw_to_repr.iter() {
            repr_to_raw[C::sym_to_usize(sym)] = raw;
        }
        Self {
            raw_to_repr,
            repr_to_raw,
        }
    }

    pub fn sym_coord(&self, state: State, sym: &Symmetries) -> C::Raw {
        for i in 0..sym.size() {
            let raw = C::from_state(&sym.conj_inv(state, i)).repr();
            if let Some(&coord) = self.raw_to_repr.get(&raw) {
                return C::pack_sym_coord(coord, i);
            }
        }
        unreachable!("a state always has a sym-coord");
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

    pub fn deserialize<Source: Read>(source: &mut Source) -> std::io::Result<Self>
    where
        for<'a> &'a [u8]: TryInto<&'a <C::Raw as FromBytes>::Bytes>,
    {
        let mut buffer = Vec::new();
        source.read_to_end(&mut buffer)?;
        let sym_to_raw = bits::deserialize_array::<C::Raw>(&buffer);
        let mut raw_to_sym = HashMap::new();
        for (i, &raw) in sym_to_raw.iter().enumerate() {
            raw_to_sym.insert(raw, C::usize_to_sym(i));
        }
        Ok(Self {
            raw_to_repr: raw_to_sym,
            repr_to_raw: sym_to_raw,
        })
    }

    pub fn serialize<Sink: Write>(&self, sink: &mut Sink) -> std::io::Result<()> {
        sink.write_all(&serialize_array(&self.repr_to_raw))
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        algebra::{
            coord::{CO, Coord, EOLR, LR},
            sym::Symmetries,
            sym_coord::SymCoordTable,
        },
        core::moves::Moves,
    };

    #[test]
    fn sym_coords() {
        fn test<C: Coord>(sym: &Symmetries) {
            let table = SymCoordTable::<C>::build(sym);
            for i in C::all_sym_coords() {
                assert_eq!(i, table.raw_to_repr[&table.repr_to_raw[C::sym_to_usize(i)]]);
            }
            for (_, s) in Moves::to_depth(3) {
                let raw_coord = C::from_state(&s).repr();
                let sym_coord = table.sym_coord(s, sym);
                assert_eq!(raw_coord, table.raw_coord(sym_coord, sym));
                let (i, j) = C::unpack_sym_coord(sym_coord);
                let ri = C::from_repr(table.repr_to_raw[C::sym_to_usize(i)]).sample_state();
                assert_eq!(raw_coord, C::from_state(&sym.conj(ri, j)).repr());
            }
        }
        let sym = Symmetries::sub16();
        test::<CO>(&sym);
        // test::<EO>(&sym); The EO coord is not compatible with those symmetries.
        test::<LR>(&sym);
        test::<EOLR>(&sym);
    }
}
