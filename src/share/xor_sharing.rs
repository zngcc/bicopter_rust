use crate::share::{BitStorage, BitVec, Sharing};
use rand::{CryptoRng, Rng, RngCore};
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::SeedableRng;


pub struct XorSharing {
    rng: ChaChaRng,
}

impl XorSharing {
    pub fn new(rng_seed: [u8; 32],) -> Self {
        Self { rng: ChaChaRng::from_seed(rng_seed),}}
    pub fn default() -> Self {
        Self {rng: ChaChaRng::from_entropy(),} //默认为随机seed
    }

}

impl Sharing for XorSharing{
    type Plain = bool; //这个没用到，但是Sharing trait里要定义这个，所以就随便写了个bool
    type ShareType = BitVec; //给类型改个名

    fn share(&mut self, input: Self::ShareType) -> [Self::ShareType; 2] {
        let input_length: usize = input.len()/8; //用的u8 先除以8了
        let rand_bits: Vec<BitStorage>= (0..input_length).map(|_| self.rng.gen_range(0..9)).collect();  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
        let rand_bits_vec = BitVec::from_vec(rand_bits); //转成BitVec
        let masked_input = input ^ &rand_bits_vec;
        [rand_bits_vec, masked_input]
    }

    fn reconstruct(&mut self, shares: [Self::ShareType; 2]) -> Self::ShareType {
        let [a, b] = shares;
        a ^ b
    }
}


#[test]
fn test_xor_share() {
    let lx: i8 = 2; //明文几个数
    let mut rng = ChaChaRng::from_entropy(); 

    let plaintext: Vec<u8> = (0..lx).map(|_| rng.gen_range(0..9)).collect(); //随机生成明文bit串
    println!("plaintext: \n {:?}", plaintext);
    let plain_vex = BitVec::from_vec(plaintext);
    println!("plain_vex: \n {}", plain_vex);

    let mut xorsharing_generator = XorSharing::default();
    let [rand_bits_vec, masked_input] = xorsharing_generator.share(plain_vex);
    println!("rand_bits_vec: \n {}; \nmasked_input: \n {}", rand_bits_vec, masked_input);

    let reconstructed_input = xorsharing_generator.reconstruct([rand_bits_vec, masked_input]);
    println!("reconstructed_vec: \n {}", reconstructed_input);

    let v = reconstructed_input.into_vec();
    println!("reconstructed_input: \n {:?}", v);
}

#[test]
fn test2() {
    use bitvec::prelude::*;

    let mut data = bitvec![u16, Msb0; 1, 0,1,1];
    println!("{:?}", data);


    let mut data2 = bitvec![u8, Msb0; 1,0,0,1];
    println!("{:?}", data2);

    data2.pop();
    println!("{:?}", data2);
    data.append(&mut data2);
    println!("{:?}", data);
    data.shift_right(1);
    println!("{:?}", data);
    data.copy_within(1 .. 2,0); //移位后补1或者0
    println!("{:?}", data);
    
}