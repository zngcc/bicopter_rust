use bicopter::share::{MyBitVec,Sharing};
use bicopter::share::add_sharing::AddSharing;

use bicopter::utils::{signtrueform_tobitvec,signtrueform_to_complement,bitvec_tosigntrueform};
use bicopter::party::{BicopterParty, BicopterParty2};

fn main() {
    let (lx, neg) = (29, false); //lx和符号位
    
    //生成随机明文
    use bicopter::utils::gen_random_u64_complement;
    let sign_trueformx = gen_random_u64_complement(lx, neg);

    //或者自定明文
    // use bicopter::utils::SignTrueform;
    // let sign_trueformx = SignTrueform::new(neg, 42);

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
    println!("ReLu(x): \n{:?}\n",  reconstructed_relu);

}




