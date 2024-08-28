use crate::share::{BitStorage,MyBitVec};
use rand::Rng;
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::SeedableRng;
use std::collections::HashMap;
use crate::utils::{bitvec_tosigntrueform, signtrue_tobitvec};
use crate::utils::bitvec_arithmetic::{bitvec_mult,bitvec_add,bitvec_sub, bitvec_opposite};
use shuffle::shuffler::Shuffler;
use shuffle::irs::Irs;

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
        let t = self.gen_randomvec(0)[0]%2;
        //let t: u64 = 0;
        //(-1)^t
        let t_neg1 = signtrue_tobitvec(if t == 0 { false } else { true }, 1);
        //random for generate t_share
        let mut randnum_t = self.gen_randombitvec(0);
        if self.party_id == 0 {
            randnum_t = bitvec_opposite(&randnum_t);
        }
        let u_ = bitvec_add(&randnum_t, &t_neg1); 
        let u_0 = bitvec_mult(&t_neg1, &self.x_share);
        //
        let v_ = bitvec_sub(
            &bitvec_add(
                &u_, 
                &bitvec_mult(&signtrue_tobitvec(false,3),&u_0)
            ),  
            &signtrue_tobitvec(false,if self.party_id ==0 {1} else {0})
        );
        self.v_store.insert(128, v_); //v*
        self.u_store.insert(128, u_); //u*
        self.u_store.insert(0, u_0.clone()); //u0
        self.abc_store.insert("t".to_string(), signtrue_tobitvec(false, t));

        self.shift_and_store(&self.x_share.clone(), lx);
        self.gen_drelu_randombitvecs(lx);
        self.gen_lxv(lx);
        self.gen_w(lx);
    }
    //右移一位
    fn rightshift1(&self, mut input: MyBitVec)->MyBitVec{
        if input.is_empty() {input}
        else {
        input.shift_right(1);
        input.copy_within(1..2,0);
        input} //开头补位 
    }
    fn rightshift_n_bit(&self, mut input: MyBitVec, shift_num: usize)->MyBitVec{
        for _i in 0..shift_num {
            input = self.rightshift1(input);
        }
        input
    }
    //移位shift_num次并存储在uStore中 
    fn shift_and_store(&mut self, input: &MyBitVec, shift_num: usize){
        //self.ShiftsStore.insert(0, input.clone()); //u0=x
        if self.party_id ==0{
            for i in 1..shift_num+1 {// shift shift_num次，每次都存一下
                let mut input2 = input.clone();
                input2 = self.rightshift_n_bit(input2,i);
            //println!("{:?}{:?}{:?}", self.party_id, i, input2);
                self.u_store.insert(i, input2.clone());}}
        else{
            for i in 1..shift_num+1 {
                let mut input2 = bitvec_opposite(&input);
                input2 = bitvec_opposite(&self.rightshift_n_bit(input2,i));
            //println!("{:?}{:?}{:?}", self.party_id, i, input2);
                self.u_store.insert(i, input2.clone());}}
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
        v_i = bitvec_sub(&v_i, &signtrue_tobitvec(false, if self.party_id ==0 {1} else {0}));
        self.v_store.insert(i_v, v_i);
    }
    //生成w0~wlx和w*
    fn gen_w(&mut self, lx: usize){
        let mut irs: Irs<usize> = Irs::<usize>::default();
        let mut shuffled_index: Vec<usize> = (0..lx+2).collect();
        //这个shuffle输入序列超过24,p0p1就不同步了
        match irs.shuffle(&mut shuffled_index, &mut self.rng.0) {
            Ok(()) => (),
            Err(e) => eprintln!("Failed to shuffle: {}", e),
        }
        //irs.shuffle(&mut shuffled_index, &mut self.Rng);
        for i in 0..lx+1 {
            let w_i =bitvec_mult(&self.get_rdrelui(i), &self.get_vi(i));
            self.w_store.insert(shuffled_index[i], w_i);
            // self.w_store.insert(i, w_i);
        }
        //w*
        self.w_store.insert(shuffled_index[lx+1], bitvec_mult(&self.get_rdrelui(128), &self.get_vi(128)));
        // self.w_store.insert(lx+1, bitvec_mult(&self.get_rdrelui(128), &self.get_vi(128)));
        println!("shuffled_index: {:?}",shuffled_index);
        //最后需要按键排序一下，这里没写
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
        for i in 0..lx+2 {
            let party0_wi = self.get_wstore0i(i);
            let party1_wi = self.get_wstore1i(i);
            let reconstructed_wi =bitvec_tosigntrueform( &bitvec_add(&party0_wi, &party1_wi));
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
fn trunc_and_reconstruct_test() {
    use crate::share::Sharing;
    use crate::utils::{signtrueform_tobitvec,signtrueform_to_complement,gen_random_u64_complement};
    use crate::share::add_sharing::AddSharing;

    let (lx, neg) = (8, false); //lx和符号位
    
    //生成随机明文
    
    let sign_trueformx = gen_random_u64_complement(lx, neg);
    //或者自定明文
    // use crate::utils::{SignTrueform
    // let sign_trueformx = SignTrueform::new(neg, 42);

    //错误位置 42 19 41 26 70 43 53 56
    println!("符号位: {:?} 原码：{:?} 补码：{:?}", sign_trueformx.get_sign(), sign_trueformx.get_trueform(),  signtrueform_to_complement(&sign_trueformx)); //符号 原码 补码
    let plain_vex = signtrueform_tobitvec(&sign_trueformx);
    println!("true_vec:\n{:?}", MyBitVec::from_vec(vec!{sign_trueformx.get_trueform()})); //原码对应的bitvec
    println!("comp_vex:\n{:?}", plain_vex);        //补码对应的bitvec

    let randseed01: [u8; 32] = [1u8;32];
    let randseed02: [u8; 32] = [2u8;32];
    let randseed12: [u8; 32] = [12u8;32];
    let mut addsharing_generator = AddSharing::default();
    let mut party0 = BicopterParty::new(0,randseed01,randseed02);
    let mut party1 = BicopterParty::new(1,randseed01,randseed12);

//DRelu
    let [rand_bits_vec, masked_input] = addsharing_generator.share(plain_vex.clone());
    //println!("rand_bits_vec: \n {}; \nmasked_input: \n {}", rand_bits_vec, masked_input);


    party0.shift_and_store(&rand_bits_vec, lx);
    party1.shift_and_store(&masked_input,lx);

    for i in 1..lx+1{
    print!("shift {} bits, ", i);
    let party0_rightshift = party0.get_ui(i);
    let party1_rightshift = party1.get_ui(i);

    let reconstructed_input = addsharing_generator.reconstruct([party0_rightshift, party1_rightshift]);
    println!("reconstructed_vec: \n {}", reconstructed_input);

    let v = bitvec_tosigntrueform(&reconstructed_input);
    println!("reconstructed_input: \n {:?}", v);


    }

}
