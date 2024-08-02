use rand::{Rng};
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::SeedableRng;
use crate::share::{BitStorage};

pub fn gen_random_u64_plaintext(lx: usize, neg: bool)->(u64, Vec<BitStorage>){
    let mut rng = ChaChaRng::from_entropy(); 
    let palintex_max: u64 = 2u64.pow(lx.try_into().unwrap());
    let x= rng.gen_range(0..palintex_max);
    let (x, plaintext) = gen_u64_plaintext(x, neg);
    (x,plaintext)
}

pub fn gen_u64_plaintext(x: u64, neg: bool) ->(u64, Vec<BitStorage>){
    if neg {
        let par_p :u128 = 2u128.pow(64);
        let plaintext = vec![(par_p-(x as u128)) as BitStorage];
        (x, plaintext)
    }
    else {
        let plaintext = vec![x as BitStorage];
        (x, plaintext)
        }
}


#[test]
fn test1(){

    let plaintext = gen_random_u64_plaintext(8, false);
    println!("plaintext: \n {:?}", plaintext);

    let plaintext = gen_random_u64_plaintext(8, true);
    println!("plaintext: \n {:?}", plaintext);


}


#[test]
fn test2(){
    let lx: u32 = 8; //明文几位
    let palintex_max: u64 = 2u64.pow(lx);
    let par_p :u128 = 2u128.pow(64);
    let palintex_min = par_p - palintex_max as u128;
    println!("par_p: \n {:?}", par_p);
    println!("palintex_min: \n {:?}", palintex_min);



    let mut rng = ChaChaRng::from_entropy(); 
    

    let plaintext: Vec<BitStorage> = (0..1).map(|_| rng.gen_range(palintex_min..par_p) as BitStorage).collect(); //随机生成明文bit串

}