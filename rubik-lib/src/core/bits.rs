use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

use num::{
    PrimInt,
    traits::{FromBytes, ToBytes},
};

pub trait Int: PrimInt + FromBytes + ToBytes + Hash + Debug + Display + Send + Sync {}

impl<T> Int for T where T: PrimInt + FromBytes + ToBytes + Hash + Debug + Display + Send + Sync {}

pub fn get<T: Int>(bitfield: T, pos: u8, count: u8) -> T {
    let one = T::one();
    let mask = (one << (count as usize)) - one;
    (bitfield >> (pos as usize)) & mask
}

pub fn bitwise_add_mod_2(a: &mut u64, b: u64) {
    let one = 0b_00001_00001_00001_00001_00001_00001_00001_00001_00001_00001_00001_00001;
    *a ^= b & one;
}

pub fn bitwise_add_mod_3(a: &mut u64, b: u64) {
    let one = 0b_00001_00001_00001_00001_00001_00001_00001_00001;
    let input_mask = one | (one << 1);
    let ai = *a & input_mask;
    let bi = b & input_mask;
    let sum = ai + bi;
    let ovfl = (sum + one) & (one << 2);
    let not_ovfl = ovfl ^ (one << 2);
    let mask = (not_ovfl >> 1) | (not_ovfl >> 2);
    *a &= !input_mask;
    *a |= sum & mask;
    *a |= (ovfl >> 2) & (sum + one);
}

pub fn bitwise_inv_mod_3(a: &mut u64) {
    let one = 0b_00001_00001_00001_00001_00001_00001_00001_00001;
    let input_mask = one | (one << 1);
    let ai = *a & input_mask;
    *a &= !input_mask;
    *a |= (ai >> 1) & one;
    *a |= (ai & one) << 1;
}

pub fn deserialize_array<T: Int>(buffer: &[u8]) -> Vec<T>
where
    for<'a> &'a [u8]: TryInto<&'a <T as FromBytes>::Bytes>,
{
    let mut array = Vec::new();
    for bytes in buffer.chunks_exact(size_of::<T>()) {
        // Safety: bytes is a slice of the exact same size as the target so it can be converted
        let bytes: &<T as FromBytes>::Bytes = unsafe { bytes.try_into().unwrap_unchecked() };
        array.push(T::from_ne_bytes(bytes));
    }
    array
}

pub fn serialize_array<T: Int>(array: &[T]) -> Vec<u8> {
    let mut buffer = Vec::new();
    for elem in array {
        buffer.extend_from_slice(elem.to_ne_bytes().as_ref());
    }
    buffer
}

#[cfg(test)]
mod tests {
    use crate::core::bits::{
        bitwise_add_mod_3, bitwise_inv_mod_3, deserialize_array, serialize_array,
    };

    #[test]
    fn add_mod_3() {
        let mut a = 0b_00000_00100_01001_01001_01110_10010;
        let b = 0b_10100_10110_01001_01110_01110_10001;
        bitwise_add_mod_3(&mut a, b);
        assert_eq!(a, 0b_00000_00110_01010_01000_01101_10000);
    }

    #[test]
    fn inv_mod_3() {
        let mut a = 0b_00000_00100_01001_01001_01110_10010;
        bitwise_inv_mod_3(&mut a);
        assert_eq!(a, 0b_00000_00100_01010_01010_01101_10001);
    }

    #[test]
    fn serialize() {
        let array: [u32; _] = [1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(&deserialize_array::<u32>(&serialize_array(&array)), &array)
    }
}
