use rand::Rng;
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::SeedableRng;
use crate::share::{CONST_P_64, BitStorage,MyBitVec};
pub mod bitvec_arithmetic;

//定义一个包含符号和原码的结构体
#[derive(Debug)]
pub struct SignTrueform{
    sign: bool,
    trueform: BitStorage,
}
impl SignTrueform {
    pub fn new(sign: bool, trueform: BitStorage)->SignTrueform{
        SignTrueform{
            sign,
            trueform,
        }
    }
    pub fn default()->SignTrueform{
        SignTrueform{
            sign: false,
            trueform: 0,
        }
    }
    pub fn get_trueform(&self)->BitStorage{
        self.trueform
    }
    pub fn get_sign(&self)->bool{
        self.sign
    }
}
//从符号和原码得到补码的bitvec
pub fn signtrue_tobitvec(sign: bool, trueform: BitStorage)->MyBitVec{
    let signtrueform = SignTrueform::new(sign, trueform);
    let bitvec=signtrueform_tobitvec(&signtrueform);
    bitvec
}
//从SignTrueform(符号和原码)得到补码的bitvec
pub fn signtrueform_tobitvec(signtrueform: &SignTrueform)->MyBitVec{
    let complement_vec = vec![signtrueform_to_complement(signtrueform)];
    let bitvec = MyBitVec::from_vec(complement_vec);
    bitvec
}
//从SignTrueform(符号和原码)得到补码的u64
pub fn signtrueform_to_complement(sign_x: &SignTrueform) ->BitStorage{
    if sign_x.sign {
        //let par_p :u128 = 2u128.pow(64);
        let complement = (CONST_P_64-(sign_x.trueform as u128)) as BitStorage;
        complement
    }
    else {
        let complement = sign_x.trueform  as BitStorage;
        complement
        }
}
//从bitvec格式的补码得到SignTrueform(符号和原码)
pub fn bitvec_tosigntrueform(complement_bitvec: &MyBitVec<BitStorage>)->SignTrueform{
    let complement_vec = complement_bitvec.clone().into_vec();
    let sign_true = vec_tou64(&complement_vec);
    sign_true
} 
//从BitStorage(u64)的vec得到SignTrueform(符号和原码)
pub fn vec_tou64(complement_vec: &Vec<BitStorage>)->SignTrueform{
    let complement = complement_vec[0];
    let sign_true = u64_tosigntrue(complement);
    sign_true
} 
//从BitStorage(u64)的补码得到SignTrueform(符号和原码)
pub fn u64_tosigntrue(complement: BitStorage)->SignTrueform{
    let par_p :u128 = CONST_P_64;
    let absolute_value = complement;
    let neg= (absolute_value as u128)>par_p/2;
    if neg {
        let x = par_p-(absolute_value as u128);
        SignTrueform::new(neg, x as BitStorage)
    }
    else {
        SignTrueform::new(neg, absolute_value)
    }
}

//生成随机BitStorage(u64)格式的原码x和补码complement
pub fn gen_random_u64_complement(lx: usize, sign: bool)->SignTrueform{
    let mut rng = ChaChaRng::from_entropy(); 
    let x_max = 2u32.pow(lx.try_into().unwrap()) as u64; //这里是x的最大值，取决于lx的大小,暂且认为不大于32位
    let x= rng.gen_range(0..x_max);
    let sign_truex: SignTrueform = SignTrueform::new(sign, x );
    sign_truex
}


#[test]
fn test1(){
    let sign_trueformx = gen_random_u64_complement(8, false);
    println!("plaintext: \n {:?}, {:?}", sign_trueformx, signtrueform_to_complement(&sign_trueformx));
    println!("plaintext: \n neg: {:?}", sign_trueformx);

    let sign_trueformx = gen_random_u64_complement(8, true);
    println!("plaintext: \n {:?}, {:?}", sign_trueformx, signtrueform_to_complement(&sign_trueformx));
    println!("plaintext: \n neg: {:?}", sign_trueformx);

    let sign_trueformx = gen_random_u64_complement(8, true);
    println!("plaintext: \n {:?}, {:?}", sign_trueformx, signtrueform_to_complement(&sign_trueformx));
    println!("plaintext: \n neg: {:?}", sign_trueformx);
    

}
