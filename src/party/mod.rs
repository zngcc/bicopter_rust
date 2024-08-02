use crate::share::{BitStorage,BitVec,Sharing};
use rand::{CryptoRng, Rng, RngCore};
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::SeedableRng;
use std::fmt::Debug;
use std::collections::HashMap;
use std::u8::MAX as u8MAX;

pub struct BicopterParty {
    PartyId: u8,
    Rng: ChaChaRng,
    ShiftsStore: HashMap<usize, BitVec>,
    RandomStore: HashMap<usize, BitVec>,
}
impl BicopterParty {
    pub fn new(id : u8,rng_seed: [u8; 32]) -> Self {
        Self { PartyId : id, Rng: ChaChaRng::from_seed(rng_seed), ShiftsStore: HashMap::new(), RandomStore: HashMap::new()}}
}

impl BicopterParty {
    pub fn get_id(&self) -> &u8 {&self.PartyId}
    pub fn rightshift1(&mut self, mut input:  BitVec)-> BitVec{
        input.shift_right(1);
        input.copy_within(1 .. 2,0); //移位后补
        input
    }
    fn genrandom(&mut self) -> BitVec{
        let rand_bits: Vec<BitStorage>= (0..1).map(|_| self.Rng.gen_range(0..9)).collect();  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
        let rand_bits_vec = BitVec::from_vec(rand_bits); //转成BitVec
        rand_bits_vec
    }
    pub fn shift_and_store(&mut self, mut input: BitVec, shift_num: usize){
        for i in 1..shift_num+1 {// shift shift_num次，每次都存一下
            input = self.rightshift1(input);
            self.ShiftsStore.insert(i, input.clone());}
    }
    pub fn get_shifted_input(&self, shift_index: usize) -> BitVec{
        self.ShiftsStore.get(&shift_index).unwrap().clone()
    }

}


#[test]
fn trunc_and_reconstruct_test() {
    use crate::share::xor_sharing::XorSharing;
    use crate::utils::gen_random_u64_plaintext;

    let (lx, neg) = (6, true); //明文几位
    let (x, plaintext) = gen_random_u64_plaintext(lx, neg);

    println!("plaintext: \n {:?} {} {:?}", x, neg, plaintext);
    let plain_vex = BitVec::from_vec(plaintext);
    println!("x: \n {}", BitVec::from_vec(vec!{x}));
    println!("plain_vex: \n {}", plain_vex);

    let randseed: [u8; 32] = [0u8;32];
    let mut xorsharing_generator = XorSharing::default();
    let mut party0 = BicopterParty::new(0,randseed);
    let mut party1 = BicopterParty::new(1,randseed);

    let [rand_bits_vec, masked_input] = xorsharing_generator.share(plain_vex);
    //println!("rand_bits_vec: \n {}; \nmasked_input: \n {}", rand_bits_vec, masked_input);


    party0.shift_and_store(rand_bits_vec, lx);
    party1.shift_and_store(masked_input,lx);

    for i in 1..lx{
    print!("shift {} bits, ", i);
    let party0_rightshift = party0.get_shifted_input(i);
    let party1_rightshift = party1.get_shifted_input(i);

    let reconstructed_input = xorsharing_generator.reconstruct([party0_rightshift, party1_rightshift]);
    println!("reconstructed_vec: \n {}", reconstructed_input);

    let v = reconstructed_input.into_vec();
    println!("reconstructed_input: \n {:?}", v);
    }

}
