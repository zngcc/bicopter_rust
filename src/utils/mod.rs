use rand::{Rng};
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::SeedableRng;
use crate::share::{BitStorage,BitVec};
pub mod bitvec_arithmetic;

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
//从符号和源码得到补码的bitvec
pub fn signtrue_tobitvec(sign: bool, trueform: BitStorage)->BitVec{
    let signtrueform = SignTrueform::new(sign, trueform);
    let bitvec=signtrueform_tobitvec(&signtrueform);
    bitvec
}
pub fn signtrueform_tobitvec(signtrueform: &SignTrueform)->BitVec{
    let complement_vec = vec![signtrueform_to_complement(signtrueform)];
    let bitvec = BitVec::from_vec(complement_vec);
    bitvec
}
//返回源码和只包含一个元素的补码vec(u64)，用于bitvec生成


//从SignTrueform(符号和源码)得到u64补码
pub fn signtrueform_to_complement(sign_x: &SignTrueform) ->u64{
    if sign_x.sign {
        let par_p :u128 = 2u128.pow(64);
        let complement = (par_p-(sign_x.trueform as u128)) as BitStorage;
        complement
    }
    else {
        let complement = sign_x.trueform  as BitStorage;
        complement
        }
}
//从bitvec的补码得到SignTrueform(符号和源码)
pub fn bitvec_tosigntrueform(complement_bitvec: &BitVec<u64>)->SignTrueform{
    let complement_vec = complement_bitvec.clone().into_vec();
    let sign_true = vec_tou64(&complement_vec);
    sign_true
} 
//从u64的vec得到SignTrueform(符号和源码)
pub fn vec_tou64(complement_vec: &Vec<u64>)->SignTrueform{
    let complement = complement_vec[0];
    let sign_true = u64_tosigntrue(complement);
    sign_true
} 
//从u64的补码得到SignTrueform(符号和源码)
pub fn u64_tosigntrue(complement: u64)->SignTrueform{
    let par_p :u128 = 2u128.pow(64);
    let absolute_value = complement;
    let neg= (absolute_value as u128)>par_p/2;
    if neg {
        let x = par_p-(absolute_value as u128);
        SignTrueform::new(neg, x as u64)
    }
    else {
        SignTrueform::new(neg, absolute_value)
    }
}


pub fn gen_random_u64_complement_vec(lx: usize, neg: bool)->(BitStorage, Vec<BitStorage>){
    let (x,complement) = gen_random_u64_complement(lx, neg);
    let complement_vec = vec![complement];
    (x, complement_vec)
}

//生成随机u64源码和补码
pub fn gen_random_u64_complement(lx: usize, neg: bool)->(BitStorage, BitStorage){
    let mut rng = ChaChaRng::from_entropy(); 
    let x_max = 2u64.pow(lx.try_into().unwrap()); //这里是x的最大值，取决于lx的大小,暂且认为不大于64位，之后要改
    let x= rng.gen_range(0..x_max);
    let complement = signtrueform_to_complement(&SignTrueform::new(neg, x ));
    (x,complement) //x是绝对值，plaintext是补码（放到了vec里）
}
#[test]
fn test1(){

    let (abs, numvec) = gen_random_u64_complement_vec(8, false);
    println!("plaintext: \n {:?}, {:?}", abs, numvec);
    let sign_true = vec_tou64(&numvec);
    println!("plaintext: \n neg: {:?}", sign_true);

    let (abs, numvec) = gen_random_u64_complement_vec(8, true);
    println!("plaintext: \n {:?}, {:?}", abs, numvec);
    let sign_true = vec_tou64(&numvec);
    println!("plaintext: \n neg: {:?}", sign_true);

    let (abs, numvec) = gen_random_u64_complement_vec(8, true);
    println!("plaintext: \n {:?}, {:?}", abs, numvec);
    let num_bitvec = BitVec::from_vec(numvec);
    //print!("length {:?}", num_bitvec.len());
    let sign_true = bitvec_tosigntrueform(&num_bitvec);
    println!("plaintext: \n neg: {:?}", sign_true);
    

}
