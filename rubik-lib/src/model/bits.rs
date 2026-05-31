use num::PrimInt;

pub trait BitField: PrimInt {}

impl<T> BitField for T where T: PrimInt {}

pub fn get<T: BitField>(bitfield: T, pos: u8, count: u8) -> T {
    let one = T::one();
    let mask = (one << (count as usize)) - one;
    (bitfield >> (pos as usize)) & mask
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
    *a &= !(one | (one << 1));
    *a |= sum & mask;
    *a |= (ovfl >> 2) & (sum + one);
}
