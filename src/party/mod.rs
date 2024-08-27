use crate::share::{BitStorage,MyBitVec,Sharing};
use rand::Rng;
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::SeedableRng;
use std::collections::HashMap;
use crate::utils::{bitvec_tosigntrueform, signtrue_tobitvec};
use crate::utils::bitvec_arithmetic::{bitvec_mult,bitvec_add,bitvec_sub};
use shuffle::shuffler::Shuffler;
use shuffle::irs::Irs;
use crate::share::add_sharing::AddSharing;

//Party 0 or Party 1
pub struct BicopterParty {
    party_id: u8,
    rng: (ChaChaRng,ChaChaRng),
    r_drelu_store: HashMap<usize, MyBitVec>,
    abc_store: HashMap<String, MyBitVec>,
    v_store: HashMap<usize, MyBitVec>, // 0是v0，然后v1 v2 ..., 128是v*
    u_store: HashMap<usize, MyBitVec>,// 0是u0，然后u1 u2 ..., 128是u*
    w_store: HashMap<usize, MyBitVec>,
    x_share: MyBitVec,
}
impl BicopterParty {
    //新建party，要求输入id和伪随机数的seed
    pub fn new(id : u8,rng_seed: [u8; 32],rng_seed2: [u8; 32]) -> Self {
        Self {
            party_id : id, 
            rng: (ChaChaRng::from_seed(rng_seed), ChaChaRng::from_seed(rng_seed2)),
            //存储DReLu计算所需的随机数（即r0...rlx）
            r_drelu_store: HashMap::new(),  
            abc_store: HashMap::new(), 
            //0:a0 1:b0 2:c0 3:d0 4:a1 5:b1 6:c1 7:d1 8:e 9:DReLu(x)'
            v_store: HashMap::new(),
            u_store: HashMap::new(),
            w_store: HashMap::new(),
            x_share: MyBitVec::new(), 
        }}

    pub fn get_partyid(&self) -> u8 {self.party_id}
    //生成 u v 和 w
    pub fn init_u_v(&mut self, x_share: MyBitVec, lx: usize){
        self.x_share = x_share;
        let xshare_signtrueform = bitvec_tosigntrueform(&self.x_share);
        let (x_sign, x_trueform) = (xshare_signtrueform.get_sign().clone(), xshare_signtrueform.get_trueform().clone());

        let t = self.gen_randomvec(0)[0]%2;
        //println!("t: {:?}", t);
        let u_ = signtrue_tobitvec(if t == 0 { false } else { true }, 1) ;
        let u_0 = signtrue_tobitvec(if t == 0 {x_sign} else {!x_sign}, x_trueform);
        let v_ = bitvec_add( &bitvec_mult(&signtrue_tobitvec(false,3),&u_0),&signtrue_tobitvec(true,1));
        self.v_store.insert(128, v_); //v*
        self.u_store.insert(128, u_); //u*
        self.u_store.insert(0, u_0); //u0
        self.abc_store.insert("t".to_string(), signtrue_tobitvec(false, t));

        self.shift_and_store(self.x_share.clone(), lx);
        self.gen_drelu_randombitvecs(lx);
        self.gen_lxv(lx);
        self.gen_w(lx);
    }
    //调用lx次gen_v，生成lx个vi (v1~vlx, 不包括v0和v*)
    fn gen_lxv(&mut self, lx: usize){
        for i in 0..lx+1 {
            self.gen_v(i, lx);
        }
    }
    //生成vi, 输入中的i_v为ui的累加起点
    fn gen_v(&mut self, i_v: usize, lx: usize){
        let mut v_i = signtrue_tobitvec(false,0);
        for i in i_v..lx+1 {
            v_i = bitvec_add(&v_i, &self.u_store.get(&(i)).unwrap());
        }
        self.v_store.insert(i_v, v_i);
    }
    //生成w0~wlx和w*
    fn gen_w(&mut self, lx: usize){
        let mut irs: Irs<usize> = Irs::<usize>::default();
        let mut shuffled_index: Vec<usize> = (0..lx+2).collect();
        match irs.shuffle(&mut shuffled_index, &mut self.rng.0) {
            Ok(()) => (),
            Err(e) => eprintln!("Failed to shuffle: {}", e),
        }
        //irs.shuffle(&mut shuffled_index, &mut self.Rng);
        for i in 0..lx+1 {
            let w_i =bitvec_mult(&self.r_drelu_store.get(&(i)).unwrap(), &self.v_store.get(&(i)).unwrap());
            self.w_store.insert(shuffled_index[i], w_i);
        }
        //w*
        self.w_store.insert(shuffled_index[lx+1], bitvec_mult(&self.r_drelu_store.get(&128).unwrap(), &self.v_store.get(&128).unwrap()));
    }
    //生成一个随机数r 并转换成BitVec
    fn gen_randomvec(&mut self, rng_id: usize) -> Vec<BitStorage>{
        let rng_to_use = match rng_id{
            0 => &mut self.rng.0,
            1 => &mut self.rng.1,
            _ => panic!("Invalid rng_id"),
        };
        let rand_bits: Vec<BitStorage>= (0..1).map(|_| rng_to_use.gen_range(1..BitStorage::MAX)).collect();  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
        rand_bits}
    fn gen_randombitvec(&mut self, rng_id: usize) -> MyBitVec{
        let rand_bits=self.gen_randomvec(rng_id);  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
        let rand_bits_vec = MyBitVec::from_vec(rand_bits); //转成BitVec
        rand_bits_vec}
    //生成多个随机数并存储到RandomStore中
    fn gen_drelu_randombitvecs(&mut self, lx: usize){
        for i in 0..lx+1 {//r0~rlx
            let rand_bits = self.gen_randombitvec(0);  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
            self.r_drelu_store.insert(i, rand_bits);
        }
        let rand_bits = self.gen_randombitvec(0);  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
        self.r_drelu_store.insert(128, rand_bits); //r*
    }

    //右移一位
    pub fn rightshift1(&mut self, mut input:  MyBitVec)-> MyBitVec{
        if input.is_empty() {return input; }
        input.shift_right(1);
        input.copy_within(1 .. 2,0); //开头补位
        input
    }
    //移位shift_num次并存储在uStore中 
    fn shift_and_store(&mut self, mut input: MyBitVec, shift_num: usize){
        //self.ShiftsStore.insert(0, input.clone()); //u0=x
        for i in 1..shift_num+1 {// shift shift_num次，每次都存一下
            input = self.rightshift1(input);
            self.u_store.insert(i, input.clone());}
    }
    pub fn get_ui(&self, shift_index: usize) -> MyBitVec{self.u_store.get(&shift_index).unwrap().clone()}
    pub fn get_vi(&self, shift_index: usize) -> MyBitVec{self.v_store.get(&shift_index).unwrap().clone()}    
    pub fn get_rdrelui(&self, random_index: usize) -> MyBitVec{self.r_drelu_store.get(&random_index).unwrap().clone()}
    pub fn get_wi(&self, shift_index: usize) -> MyBitVec{self.w_store.get(&shift_index).unwrap().clone()}
    pub fn get_wstore(&self) -> HashMap<usize, MyBitVec>{self.w_store.clone()}
    // _s代表是share, _in代表是收到的
    pub fn gen_abc(&mut self){
        //println!("party {} gen abc:", self.party_id);
        if self.party_id ==0{        
            for i in ["a_s", "b_s", "c_s"]{
                let rand_bits = self.gen_randombitvec(1);  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
                //println!("{}: {}", i, rand_bits);
                self.abc_store.insert(i.to_string(), rand_bits);
        }}
        else if self.party_id ==1{        
            for i in ["a_s", "b_s"]{
                let rand_bits = self.gen_randombitvec(1);  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
                //println!("{}: {}", i, rand_bits);
                self.abc_store.insert(i.to_string(), rand_bits);
        }}
        let d_s = bitvec_sub(&self.x_share, &self.abc_store.get("a_s").unwrap().clone());
        self.abc_store.insert("d_s".to_string(), d_s);
    }
    pub fn get_d(&self) -> MyBitVec{self.get_abc("d_s")}
    pub fn get_relu(&self) -> MyBitVec{self.abc_store.get("relu_s").unwrap().clone()}
    pub fn receive_d(&mut self, d_in: MyBitVec) {self.receive_abc("d_in".to_string(), d_in);}
    pub fn receive_c(&mut self, c_s: MyBitVec) {self.receive_abc("c_s".to_string(), c_s);}
    pub fn receive_e(&mut self, e: MyBitVec) {self.receive_abc("e".to_string(), e);}
    pub fn receive_abc(&mut self, index: String, abc_in: MyBitVec){self.abc_store.insert(index, abc_in);}
    pub fn get_abc(&self, index: &str) -> MyBitVec{self.abc_store.get(index).unwrap().clone()}
    pub fn get_xshare(&self) -> MyBitVec{self.x_share.clone()} //记得删掉 并把上面的get_abc变成private
    pub fn compute_relu(&mut self){
        //println!("party {} compute relu:", self.party_id);
        //relu = (1-2t)
        let t = self.get_abc("t");
        let d_s = self.get_abc("d_s");
        let d = bitvec_add(&self.get_abc("d_s"), &self.get_abc("d_in"));
        let e = self.get_abc("e");
        let a_s = self.get_abc("a_s");
        let b_s = self.get_abc("b_s");
        let c_s = self.get_abc("c_s");

        //println!("t, d, e: \n{}\n{}\n{}", t, d,e);
        let relu_s = bitvec_add(
        &bitvec_mult(
            &bitvec_sub(&signtrue_tobitvec(false, 1), &bitvec_add(&t, &t)),
            &bitvec_add(
                &bitvec_add(
                    &bitvec_mult(&d_s, &e), 
                    &bitvec_mult(&d, &b_s)), 
                &bitvec_add(
                    &bitvec_mult(&e, &a_s), 
                    &c_s)
                )
            ),
            &bitvec_mult(&t, &self.x_share)
        );
        self.abc_store.insert("relu_s".to_string(), relu_s);
    }


}

pub struct BicopterParty2 {
    party_id: u8,
    w_store0: HashMap<usize, MyBitVec>,
    w_store1: HashMap<usize, MyBitVec>,
    abc_store: HashMap<String, MyBitVec>,
    rng: (ChaChaRng, ChaChaRng),
}
impl BicopterParty2 {
    pub fn new(rng_seed0: [u8; 32], rng_seed1: [u8; 32]) -> Self {
        Self {
            party_id : 2, 
            w_store0: HashMap::new(), 
            w_store1: HashMap::new(),
            abc_store: HashMap::new(), 
            rng: (ChaChaRng::from_seed(rng_seed0), ChaChaRng::from_seed(rng_seed1)), 
        }}
    pub fn get_partyid(&self) -> u8 {self.party_id}
    //接收w0 w1
    pub fn receive_w0(&mut self, w0_in: HashMap<usize, MyBitVec>){self.w_store0 = w0_in;}
    pub fn receive_w1(&mut self, w1_in: HashMap<usize, MyBitVec>){self.w_store1 = w1_in;}
    pub fn get_wstore0i(&self, shift_index: usize) -> MyBitVec{self.w_store0.get(&shift_index).unwrap().clone()}
    pub fn get_wstore1i(&self, shift_index: usize) -> MyBitVec{self.w_store1.get(&shift_index).unwrap().clone()}
    pub fn get_wstore0(&self) -> HashMap<usize, MyBitVec>{self.w_store0.clone()}
    pub fn get_wstore1(&self) -> HashMap<usize, MyBitVec>{self.w_store1.clone()}
    //计算并检查w中是否有0
    pub fn check_w(&mut self, lx: usize) ->MyBitVec {
        let mut addsharing_generator = AddSharing::default();
        for i in 0..lx+2 {
            let party0_wi = self.get_wstore0i(i);
            let party1_wi = self.get_wstore1i(i);
            let reconstructed_wi =bitvec_tosigntrueform( &addsharing_generator.reconstruct([party0_wi, party1_wi]));
            if reconstructed_wi.get_trueform() == 0 {
                let drelu_p = signtrue_tobitvec(false,0);
                self.abc_store.insert("drelu_p".to_string(), drelu_p.clone());
                return drelu_p
            }
        }
        //没有零 不是正数 符号位为1
        let drelu_p = signtrue_tobitvec(false,1);
        self.abc_store.insert("drelu_p".to_string(), drelu_p.clone());
        drelu_p
    }
    fn gen_randomvec(&mut self, rng_id: usize) -> Vec<BitStorage>{
        let rng_to_use = match rng_id{
            0 => &mut self.rng.0,
            1 => &mut self.rng.1,
            _ => panic!("Invalid rng_id"),
        };
        let rand_bits: Vec<BitStorage>= (0..1).map(|_| rng_to_use.gen_range(1..BitStorage::MAX)).collect();  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
        rand_bits}
    fn gen_randombitvec(&mut self, rng_id: usize) -> MyBitVec{
        let rand_bits=self.gen_randomvec(rng_id);  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
        let rand_bits_vec = MyBitVec::from_vec(rand_bits); //转成BitVec
        rand_bits_vec}


    //生成多个随机数并存储到RandomStore中
    pub fn gen_abc(&mut self){
        //a0 b0 c0
        for i in ["a0","b0","c0"]{ 
            let rand_bits = self.gen_randombitvec(0);  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
            //println!("{}: {}", i, rand_bits);
            self.abc_store.insert(i.to_string(), rand_bits);
        }
        //a1 b1
        for i in ["a1","b1"]{ 
            let rand_bits = self.gen_randombitvec(1);  //生成随机比特（这样比较慢，之后改成rand::distributions::Uniform）
            //println!("{}: {}", i, rand_bits);
            self.abc_store.insert(i.to_string(), rand_bits);
        }
        //c1 = (a0+a1)(b0+b1) - c0
        let c1 = bitvec_sub(
        &bitvec_mult(
            &bitvec_add(&self.abc_store.get("a0").unwrap().clone(), &self.abc_store.get("a1").unwrap().clone()),
            &bitvec_add(&self.abc_store.get("b0").unwrap().clone(), &self.abc_store.get("b1").unwrap().clone())
                ),
        &self.abc_store.get("c0").unwrap().clone()
        );
        self.abc_store.insert("c1".to_string(), c1);
    }
    pub fn gen_e(&mut self){
        let e = bitvec_sub(
        &self.abc_store.get("drelu_p").unwrap().clone(),
        &bitvec_add(
            &self.abc_store.get("b0").unwrap().clone(), 
            &self.abc_store.get("b1").unwrap().clone()
            )
        );
        self.abc_store.insert("e".to_string(), e);
    }
    pub fn get_abc(&self, index: String) -> MyBitVec{self.abc_store.get(&index).unwrap().clone()}
    pub fn get_e(&self) -> MyBitVec{self.get_abc("e".to_string())}
    pub fn get_c1(&self) -> MyBitVec{self.get_abc("c1".to_string())}
}


#[test]
fn test_relu(){


}

/*
fn test_abc(){
    use crate::utils::{gen_random_u64_complement_vec,signtrueform_tobitvec,signtrueform_to_complement};

    let (lx, neg) = (8, true); //lx和符号位
    
    //生成随机明文
    let sign_trueformx = gen_random_u64_complement_vec(lx, neg);
    //或者自定明文
    //use bicopter::utils::SignTrueform;
    //let sign_trueformx = SignTrueform::new(neg, 15);


    println!("正数?: {} 原码：{:?} 补码：{:?}", !neg, sign_trueformx.get_trueform(),  signtrueform_to_complement(&sign_trueformx)); //符号 原码 补码
    let plain_vex = signtrueform_tobitvec(&sign_trueformx);
    println!("x_vec: \n {}", BitVec::from_vec(vec!{sign_trueformx.get_trueform()})); //原码对应的bitvec
    println!("plain_vex: \n {}", plain_vex);        //补码对应的bitvec

    let randseed01: [u8; 32] = [1u8;32];
    let randseed02: [u8; 32] = [2u8;32];
    let randseed12: [u8; 32] = [12u8;32];
    let mut xorsharing_generator = XorSharing::default();
    let mut party0 = BicopterParty::new(0,randseed01,randseed02);
    let mut party1 = BicopterParty::new(1,randseed01,randseed12);
    let mut party2 = BicopterParty2::new(randseed02,randseed12);
    
    //0:[a] 1:[b] 2:[c] 3:[d] 7:d_receive 8:e 9:DReLu(x)'
    party0.gen_abc();
    party1.gen_abc();
    let b0 = party0.get_abc(1);
    //println!("b0: {:?} \n", b0);
    let b1 = party1.get_abc(1);
    //println!("b1: {:?} \n", b1);
    //0:a0 1:b0 2:c0 3:d0 4:a1 5:b1 6:c1 7:d1 8:e 9:DReLu(x)'
    party2.gen_abc();
    println!("bb: \n");
    //println!("{:?} \n", party2.abc_store.clone());
    let b02=party2.get_abc(1);
    let b12=party2.get_abc(5);
    println!("b0: \n{:?} \nb02: \n{:?} \n", b0,b02);
    //assert_eq!(b0,b02);
    println!("b1: \n{:?} \nb12: \n{:?} \n", b1,b12);
    //assert_eq!(b1,b12);



}
*/


/*
#[test]
fn check_protocol(){
    use crate::share::xor_sharing::XorSharing;
    use crate::utils::{gen_random_u64_complement_vec,signtrueform_tobitvec,signtrueform_to_complement};

    let (lx, neg) = (8, true); //lx和符号位
    
    let sign_trueformx = gen_random_u64_complement_vec(lx, neg);

    println!("正数?: {} 原码：{:?} 补码：{:?}", !neg, sign_trueformx.get_trueform(),  signtrueform_to_complement(&sign_trueformx)); //符号 原码 补码
    let plain_vex = signtrueform_tobitvec(&sign_trueformx);
    println!("x_vec: \n {}", BitVec::from_vec(vec!{sign_trueformx.get_trueform()})); //原码对应的bitvec
    println!("plain_vex: \n {}", plain_vex);        //补码对应的bitvec
   

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
 */


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

/*
use bicopter::share::{MyBitVec,Sharing};
use bicopter::share::add_sharing::AddSharing;
use bicopter::utils::{gen_random_u64_complement_vec,signtrueform_tobitvec,signtrueform_to_complement,bitvec_tosigntrueform,signtrue_tobitvec};
use bicopter::party::{BicopterParty, BicopterParty2};

fn main() {
    let (lx, neg) = (8, false); //lx和符号位
    
    //生成随机明文
    let sign_trueformx = gen_random_u64_complement_vec(lx, neg);
    //或者自定明文
    //use bicopter::utils::SignTrueform;
    //let sign_trueformx = SignTrueform::new(neg, 15);


    println!("正数?: {} 原码：{:?} 补码：{:?}", !neg, sign_trueformx.get_trueform(),  signtrueform_to_complement(&sign_trueformx)); //符号 原码 补码
    let plain_vex = signtrueform_tobitvec(&sign_trueformx);
    println!("true_vec: \n {}", MyBitVec::from_vec(vec!{sign_trueformx.get_trueform()})); //原码对应的bitvec
    println!("comp_vex: \n {}", plain_vex);        //补码对应的bitvec

    let randseed01: [u8; 32] = [1u8;32];
    let randseed02: [u8; 32] = [2u8;32];
    let randseed12: [u8; 32] = [12u8;32];
    let mut addsharing_generator = AddSharing::default();
    let mut party0 = BicopterParty::new(0,randseed01,randseed02);
    let mut party1 = BicopterParty::new(1,randseed01,randseed12);
    let mut party2 = BicopterParty2::new(randseed02,randseed12);

//DRelu
    let [rand_bits_vec, masked_input] = addsharing_generator.share(plain_vex.clone());
    party0.init_u_v(rand_bits_vec, lx);
    party1.init_u_v(masked_input,lx);
    println!("生成u v w");

    
    party2.receive_w0(party0.get_wstore());
    party2.receive_w1(party1.get_wstore());
    println!("Party2接收w");

    let is_positive = party2.check_w(lx);
    println!("DReLU?: {:?}", bitvec_tosigntrueform(&is_positive).get_trueform());
//ReLU
    println!("party0 生成 a0 b0 c0 d0, party1 生成 a1 b1 d1");
    party0.gen_abc();
    party1.gen_abc();
    println!("传输d0 d1");
    let d0 = party0.get_d();
    let d1 = party1.get_d();
    party1.receive_d(d0);
    party0.receive_d(d1);


    println!("party2 生成 a0 b0 c0, a1 b1, c1, e");
    party2.gen_abc();
    party2.gen_e();
    let e = party2.get_e();
    let c1 = party2.get_c1();
    println!("传输 e, c1");
    party0.receive_e(e.clone());
    party1.receive_e(e);
    party1.receive_c(c1);

    party0.compute_relu();
    party1.compute_relu();
    let relu0 = party0.get_relu();
    let relu1 = party1.get_relu();
    let reconstructed_relu = addsharing_generator.reconstruct([relu0, relu1]);
    println!("ReLu(x): \n{:?}",  reconstructed_relu);
    

    use bicopter::utils::bitvec_arithmetic::{bitvec_add, bitvec_mult,bitvec_sub,bitvec_opposite};

    // println!("d: {:?}", bitvec_add(&party0.get_d(), &party1.get_d()));
    // let d_ = bitvec_sub(
    //     &plain_vex,
    //     &bitvec_add(
    //         &party0.get_abc("a_s"), 
    //         &party1.get_abc("a_s")
    //     )
    // );
    // println!("d: {:?}", d_);

    // let c = bitvec_add(&party0.get_abc("c_s".to_string()), &party1.get_abc("c_s".to_string()));
    // println!("c: {:?}", c);
    // let c2 = bitvec_mult(
    //     &bitvec_add(&party0.get_abc("a_s".to_string()), &party1.get_abc("a_s".to_string())), 
    //     &bitvec_add(&party0.get_abc("b_s".to_string()), &party1.get_abc("b_s".to_string())), 
    // );
    // println!("c: {:?}", c2);

    // let dr = bitvec_add(
    //     &bitvec_add(&party0.get_abc("b_s".to_string()), &party1.get_abc("b_s".to_string())), 
    //     &party0.get_abc("e".to_string())
    // );
    // println!("dr: {:?}", dr);

    let t = party0.get_abc("t");
    let d = bitvec_add(&party0.get_abc("d_s"), &party0.get_abc("d_in"));
    let e = party0.get_abc("e");
    let a_0 = party0.get_abc("a_s");
    let b_0 = party0.get_abc("b_s");
    let c_0 = party0.get_abc("c_s");

    println!("t, d, e: \n{}\n{}\n{}", t, d,e);

    let sub1_2t = bitvec_sub(&signtrue_tobitvec(false, 1), &bitvec_add(&t, &t));
    println!("sub1_2t: {}", sub1_2t);
    let de = bitvec_mult(&d, &e);
    let db0=bitvec_mult(&d, &b_0);
    let ea0 = bitvec_mult(&e, &a_0);
    let tx0 = bitvec_mult(&t, &party0.get_xshare());

    // println!("{:?}\n{:?}\n", party1.get_abc("t"), t);
    // println!("{:?}\n{:?}\n", bitvec_add(&party1.get_abc("d_s"), &party1.get_abc("d_in")), d);
    // println!("{:?}\n{:?}\n", party1.get_abc("e"), e);
    let a_1 = party1.get_abc("a_s");
    let b_1 = party1.get_abc("b_s");
    let c_1 = party1.get_abc("c_s");

    let db1 = bitvec_add(&d, &b_1);
    let ea1 = bitvec_mult(&e, &a_1);
    let tx1 = bitvec_mult(&t, &party1.get_xshare());
    println!("{:?}\n{:?}\n", plain_vex, bitvec_add(&tx0, &tx1));
    let de = bitvec_mult(&d, &e);
    let x_a_dr_b = bitvec_mult(&bitvec_sub(&plain_vex, &bitvec_add(&a_0, &a_1)), &bitvec_opposite(&bitvec_add(&b_0, &b_1)));
    println!("{:?}\n{:?}\n", de, x_a_dr_b);
    let db = bitvec_mult(&d, &bitvec_add(&b_0, &b_1));
    let db2 = bitvec_add(&bitvec_mult(&d, &b_0), &bitvec_mult(&d, &b_1));
    let x_ab= bitvec_mult(&bitvec_sub(&plain_vex, &bitvec_add(&a_0, &a_1)), &bitvec_add(&b_0, &b_1));
    println!("{:?}\n{:?}\n{:?}\n", db, db2,x_ab);



}




*/