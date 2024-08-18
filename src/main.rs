use bicopter::share::{BitVec,Sharing};
use bicopter::share::xor_sharing::XorSharing;
use bicopter::utils::{gen_random_u64_complement_vec,signtrueform_tobitvec,signtrueform_to_complement};
use bicopter::party::{BicopterParty, BicopterParty2};

fn main() {
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
    println!("Party2接收w");

    let is_positive = party2.check_w(lx);
    println!("正数?: {}", is_positive);
}




