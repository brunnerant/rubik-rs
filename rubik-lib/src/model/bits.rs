use std::ops::{BitAnd, Shl, Shr, Sub};

pub fn get<T>(bitfield: T, pos: u8, count: u8) -> T
where
    T: Copy
        + Shr<u8, Output = T>
        + Shl<u8, Output = T>
        + From<u8>
        + BitAnd<Output = T>
        + Sub<Output = T>,
{
    let one = T::from(1);
    let mask = (one << count) - one;
    (bitfield >> pos) & mask
}
