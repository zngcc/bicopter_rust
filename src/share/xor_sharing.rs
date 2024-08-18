use crate::share::{BitStorage, BitVec, Sharing};
use rand::Rng;
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
    type Plain = bool; //这个没用到
    type ShareType = BitVec;
    //利用随机数生成share
    fn share(&mut self, input: Self::ShareType) -> [Self::ShareType; 2] {
        let rand_bits: Vec<BitStorage>= (0..1).map(|_| self.rng.gen_range(0..BitStorage::MAX)).collect();  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
        let rand_bits_vec = BitVec::from_vec(rand_bits); //转成BitVec
        let masked_input = input ^ &rand_bits_vec;
        [rand_bits_vec, masked_input]
    }
    //从share恢复明文
    fn reconstruct(&mut self, shares: [Self::ShareType; 2]) -> Self::ShareType {
        let [a, b] = shares;
        a ^ b
    }
}


#[test]
fn test_xor_share() {
    let lx: u32 = 8; //明文几位
    let palintex_max: u64 = 2u64.pow(lx);
    let mut rng = ChaChaRng::from_entropy(); 
    
    let plaintext: Vec<BitStorage> = (0..1).map(|_| rng.gen_range(0..palintex_max)).collect(); //随机生成明文bit串
    //et plaintext: Vec<BitStorage> = vec![storageMAX] ; //明文
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
