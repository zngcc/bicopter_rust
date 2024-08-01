use bitvec::prelude::BitStore;
use rand::{Fill};
use crate::share::{BitVec};
pub fn rand_bitvec<T, R>(size: usize, rng: &mut R) -> BitVec<T>
where
    T: BitStore + Copy,
    [T]: Fill,
    R: CryptoRng + Rng,
{
    let bitstore_items = div_ceil(size, mem::size_of::<T>());
    let mut buf = vec![T::ZERO; bitstore_items];
    rng.fill(&mut buf[..]);
    let mut bv = BitVec::from_vec(buf);
    bv.truncate(size);
    bv
}