use crate::share::{BitStorage,BitVec,Sharing,CONST_P};
use rand::{CryptoRng, Rng, RngCore};
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::SeedableRng;
use std::fmt::Debug;
use std::collections::HashMap;
use std::u8::MAX as u8MAX;
use crate::utils::{SignTrueform,bitvec_tosigntrueform, signtrue_tobitvec};
use crate::utils::bitvec_arithmetic::{bitvec_mult,bitvec_add};
use shuffle::shuffler::Shuffler;
use shuffle::irs::Irs;
use crate::share::xor_sharing::XorSharing;
pub struct BicopterParty {
    PartyId: u8,
    Rng: ChaChaRng,
    //ShiftsStore: HashMap<usize, BitVec>,
    rStore: HashMap<usize, BitVec>,
    vStore: HashMap<usize, BitVec>, //0是v*, 1是v_0，然后才是v1 v2 ...
    uStore: HashMap<usize, BitVec>,//0是u*, 1是u_0，然后才是u1 u2 ...
    wStore: HashMap<usize, BitVec>,//0是u*, 1是u_0，然后才是u1 u2 ...
    xShare: BitVec,
    t: BitStorage,
}
impl BicopterParty {
    pub fn new(id : u8,rng_seed: [u8; 32]) -> Self {
        Self {
            PartyId : id, 
            Rng: ChaChaRng::from_seed(rng_seed), 
            //ShiftsStore: HashMap::new(), 
            rStore: HashMap::new(), 
            vStore: HashMap::new(),
            uStore: HashMap::new(),
            wStore: HashMap::new(),
            xShare: BitVec::new(), 
            t: 0,
        }}

    pub fn get_partyid(&self) -> u8 {self.PartyId}

    pub fn init_u_v(&mut self, x_share: BitVec, lx: usize){
        self.xShare = x_share;
        let xshare_signtrueform = bitvec_tosigntrueform(&self.xShare);
        let (x_sign, x_trueform) = (xshare_signtrueform.get_sign().clone(), xshare_signtrueform.get_trueform().clone());

        self.t = self.gen_randomvec()[0];
        let u_ = signtrue_tobitvec(if self.t % 2 == 0 { false } else { true }, 1) ; //这里应该是-1^{t}，暂时先这样
        let u_0 = signtrue_tobitvec(if self.t % 2 == 0 {x_sign} else {!x_sign}, x_trueform);
        let v_ = bitvec_add( &bitvec_mult(&signtrue_tobitvec(false,3),&u_0),&signtrue_tobitvec(true,1));
        self.vStore.insert(128, v_);
        self.uStore.insert(128, u_);
        self.uStore.insert(0, u_0);

        self.shift_and_store(self.xShare.clone(), lx);
        self.gen_randombitvecs(lx);
        self.gen_lxv(lx);
        self.gen_w(lx);
    }
    fn gen_lxv(&mut self, lx: usize){
        for i in 0..lx+1 {
            self.gen_v(i, lx);
        }
    }
    fn gen_v(&mut self, i_v: usize, lx: usize){
        let mut v_i = signtrue_tobitvec(false,0);
        for i in i_v..lx+1 {
            v_i = bitvec_add(&v_i, &self.uStore.get(&(i)).unwrap());
        }
        self.vStore.insert(i_v, v_i);
    }
    fn gen_w(&mut self, lx: usize){
        let mut irs: Irs<usize> = Irs::<usize>::default();
        let mut shuffled_index: Vec<usize> = (0..lx+2).collect();
        irs.shuffle(&mut shuffled_index, &mut self.Rng);
        for i in 0..lx+1 {
            let w_i =bitvec_mult(&self.rStore.get(&(i)).unwrap(), &self.vStore.get(&(i)).unwrap());
            self.wStore.insert(shuffled_index[i], w_i);
        }
        self.wStore.insert(shuffled_index[lx+1], bitvec_mult(&self.rStore.get(&128).unwrap(), &self.vStore.get(&128).unwrap()));
    }
    //生成一个随机数r 并转换成BitVec
    fn gen_randomvec(&mut self) -> Vec<BitStorage>{
        let rand_bits: Vec<BitStorage>= (0..1).map(|_| self.Rng.gen_range(1..BitStorage::MAX)).collect();  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
        rand_bits}
    fn gen_randombitvec(&mut self) -> BitVec{
        let rand_bits=self.gen_randomvec();  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
        let rand_bits_vec = BitVec::from_vec(rand_bits); //转成BitVec
        rand_bits_vec}
    //生成多个随机数并存储到RandomStore中
    fn gen_randombitvecs(&mut self, lx: usize){
        for i in 0..lx+1 {//r0~rlx
            let rand_bits = self.gen_randombitvec();  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
            self.rStore.insert(i, rand_bits);
        }
        let rand_bits = self.gen_randombitvec();  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
        self.rStore.insert(128, rand_bits); //r*
    }

    fn rightshift1(&mut self, mut input:  BitVec)-> BitVec{
        if input.is_empty() {return input; }
        input.shift_right(1);
        input.copy_within(1 .. 2,0); //移位后补
        input
    }
    pub fn shift_and_store(&mut self, mut input: BitVec, shift_num: usize){
        //self.ShiftsStore.insert(0, input.clone()); //u0=x
        for i in 1..shift_num+1 {// shift shift_num次，每次都存一下
            input = self.rightshift1(input);
            self.uStore.insert(i, input.clone());}
    }
    pub fn get_ui(&self, shift_index: usize) -> BitVec{
        self.uStore.get(&shift_index).unwrap().clone()
    }
    pub fn get_vi(&self, shift_index: usize) -> BitVec{
        self.vStore.get(&shift_index).unwrap().clone()
    }    
    pub fn get_ri(&self, random_index: usize) -> BitVec{
        self.rStore.get(&random_index).unwrap().clone()
    }
    pub fn get_wi(&self, shift_index: usize) -> BitVec{
        self.wStore.get(&shift_index).unwrap().clone()
    }
    pub fn get_wstore(&self) -> HashMap<usize, BitVec>{
        self.wStore.clone()
    }

}

pub struct BicopterParty2 {
    PartyId: u8,
    wStore0: HashMap<usize, BitVec>,//0是u*, 1是u_0，然后才是u1 u2 ...
    wStore1: HashMap<usize, BitVec>,//0是u*, 1是u_0，然后才是u1 u2 ...
}
impl BicopterParty2 {
    pub fn new() -> Self {
        Self {
            PartyId : 2, 
            wStore0: HashMap::new(), 
            wStore1: HashMap::new(),
        }}
    pub fn receive_w0(&mut self, w0_in: HashMap<usize, BitVec>){
            self.wStore0 = w0_in;
        }
    pub fn receive_w1(&mut self, w1_in: HashMap<usize, BitVec>){
            self.wStore1 = w1_in;
    }
    pub fn get_wstore0i(&self, shift_index: usize) -> BitVec{self.wStore0.get(&shift_index).unwrap().clone()}
    pub fn get_wstore1i(&self, shift_index: usize) -> BitVec{self.wStore1.get(&shift_index).unwrap().clone()}
    pub fn get_wstore0(&self) -> HashMap<usize, BitVec>{self.wStore0.clone()}
    pub fn get_wstore1(&self) -> HashMap<usize, BitVec>{self.wStore1.clone()}

    pub fn check_w(&self, lx: usize) ->bool {
        let mut xorsharing_generator = XorSharing::default();
        for i in 0..lx+2 {
            let party0_wi = self.get_wstore0i(i);
            let party1_wi = self.get_wstore1i(i);
            //println!("w: \n {:?}, {:?}", party0_wi, party1_wi);
            let reconstructed_wi =bitvec_tosigntrueform( &xorsharing_generator.reconstruct([party0_wi, party1_wi]));
            //println!("i, reconstructed_w: \n {}, {:?}", i, reconstructed_wi);
            if reconstructed_wi.get_trueform() == 0 {
                return true;
            }
        }
        false
    }

}

#[test]
fn check_protocol(){
    use crate::share::xor_sharing::XorSharing;
    use crate::utils::gen_random_u64_complement_vec;

    let (lx, neg) = (8, false); //lx长度和符号
    let (x, plaintext) = gen_random_u64_complement_vec(lx, neg);

    println!("plaintext: \n {} {:?}  {:?}", neg, x,  plaintext[0]); //符号 原码 补码
    let plain_vex = BitVec::from_vec(plaintext);
    println!("x_vec: \n {}", BitVec::from_vec(vec!{x})); //原码vec
    println!("plain_vex: \n {}", plain_vex);        //补码vec

    let randseed: [u8; 32] = [0u8;32];
    let mut xorsharing_generator = XorSharing::default();
    let mut party0 = BicopterParty::new(0,randseed);
    let mut party1 = BicopterParty::new(1,randseed);

    let [rand_bits_vec, masked_input] = xorsharing_generator.share(plain_vex);
    party0.init_u_v(rand_bits_vec, lx);
    party1.init_u_v(masked_input,lx);
    println!("生成u v w");

    let mut party2 = BicopterParty2::new();
    party2.receive_w0(party0.get_wstore());
    party2.receive_w1(party1.get_wstore());
    println!("Party2接受w");
    //println!("0 {:?}", party2.get_wstore0i(1));
    //println!("1 {:?}", party2.get_wstore1i(1));

    let positive = party2.check_w(lx);
    println!("是否为正数 {}", positive);
}



/* 
#[test]
fn check_w(){
    use crate::share::xor_sharing::XorSharing;
    use crate::utils::gen_random_u64_complement_vec;

    let (lx, neg) = (8, false); //明文几位
    let (x, plaintext) = gen_random_u64_complement_vec(lx, neg);

    println!("plaintext: \n {:?} {} {:?}", x, neg, plaintext);
    let plain_vex = BitVec::from_vec(plaintext);
    println!("x: \n {}", BitVec::from_vec(vec!{x}));
    println!("plain_vex: \n {}", plain_vex);

    let randseed: [u8; 32] = [0u8;32];
    let mut xorsharing_generator = XorSharing::default();
    let mut party0 = BicopterParty::new(0,randseed);
    let mut party1 = BicopterParty::new(1,randseed);

    let [rand_bits_vec, masked_input] = xorsharing_generator.share(plain_vex);
    party0.init_u_v(rand_bits_vec, lx);
    party1.init_u_v(masked_input,lx);

    for i in 0..lx+2{
    print!("shift {} bits:\n", i);
    let party0_w = party0.get_wi(i);
    let party1_w = party1.get_wi(i);

    let reconstructed_w = xorsharing_generator.reconstruct([party0_w, party1_w]);
    println!("{}", reconstructed_w);

    let w = bitvec_tosigntrueform(&reconstructed_w);
    println!("{:?}\n", w);
    }

}

#[test]
fn test_shffle(){
    let mut rng = ChaChaRng::from_entropy(); 
    let mut irs = Irs::<u32>::default();
    let mut input = (0..10).collect();
    println!("{:?}", input);
    irs.shuffle(&mut input, &mut rng);
    println!("{:?}{:?}", input, input[1]);
}
#[test]
fn trunc_and_reconstruct_test() {
    use crate::share::xor_sharing::XorSharing;
    use crate::utils::gen_random_u64_complement_vec;

    let (lx, neg) = (6, true); //明文几位
    let (x, plaintext) = gen_random_u64_complement_vec(lx, neg);

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

    for i in 1..lx+1{
    print!("shift {} bits, ", i);
    let party0_rightshift = party0.get_ui(i);
    let party1_rightshift = party1.get_ui(i);

    let reconstructed_input = xorsharing_generator.reconstruct([party0_rightshift, party1_rightshift]);
    println!("reconstructed_vec: \n {}", reconstructed_input);

    let v = bitvec_tosigntrueform(&reconstructed_input);
    println!("reconstructed_input: \n {:?}", v);


    }

}

#[test]
fn random_store_test() {
    let randseed: [u8; 32] = [0u8;32];
    let mut party0 = BicopterParty::new(0,randseed);
    let mut party1 = BicopterParty::new(1,randseed);
    party0.gen_randombitvecs(5);
    party1.gen_randombitvecs(5);
    for i in 0..5 {
        println!("random {}: \n {} \n {}", i, party0.get_ri(i), party1.get_ri(i));
    }
}
    */
