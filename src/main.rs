use Bicopter::share::{BitVec,Sharing};
use Bicopter::share::xor_sharing::XorSharing;
use Bicopter::utils::gen_random_u64_complement_vec;
use Bicopter::party::{BicopterParty, BicopterParty2};

fn main() {
    let (lx, neg) = (8, true); //lx和符号位
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
    println!("Party2接收w");

    let is_positive = party2.check_w(lx);
    println!("是否为正数 {}", is_positive);
}




