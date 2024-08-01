use crate::share::{BitStorage,BitVec};
use rand::{CryptoRng, Rng, RngCore};
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::SeedableRng;
use std::fmt::Debug;

pub struct BicopterParty {
    PartyId: u8,
    rng: ChaChaRng,
}



impl BicopterParty {

    pub fn new(id : u8) -> Self {
        Self { PartyId : id, rng: ChaChaRng::from_entropy(),}}
    
    fn rightshift(&mut self, mut input:  BitVec)-> BitVec{
        input.shift_right(1);
        input.copy_within(1 .. 2,0); //移位后补
        input
    }
    fn genrandom(&mut self) -> BitVec{
        let rand_bits: Vec<BitStorage>= (0..1).map(|_| self.rng.gen_range(0..9)).collect();  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
        let rand_bits_vec = BitVec::from_vec(rand_bits); //转成BitVec
        rand_bits_vec
    }





}