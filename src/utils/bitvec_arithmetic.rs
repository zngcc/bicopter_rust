use crate::share::{BitStorage, MyBitVec,CONST_P};
use crate::utils::{bitvec_tosigntrueform,SignTrueform,signtrueform_tobitvec};
//use std::ops::{Add, Mul};
//bitvec格式的补码加法
pub fn bitvec_add(lhs: & MyBitVec, rhs: &MyBitVec) -> MyBitVec {
    fn bitvec_add_self(lhs: &mut MyBitVec, rhs: &MyBitVec) {
        fn add_with_carry(a: bool, b: bool, carry: bool) -> (bool, bool) {
            let result = u8::from(a) + u8::from(b) + u8::from(carry);
            ((result & 0b01) != 0, (result & 0b10) != 0)
        }
    
        let mut carry = false;
        let extended_rhs = rhs.iter().by_vals().rev().chain(core::iter::repeat(false));
        for (mut lhs_bit, rhs_bit) in lhs.iter_mut().rev().zip(extended_rhs) {
            (*lhs_bit, carry) = add_with_carry(*lhs_bit, rhs_bit, carry);
        }
    }

    let mut result = lhs.clone();
    bitvec_add_self(&mut result, &rhs);
    result
}


//bitvec格式的补码乘法
pub fn bitvec_mult(num_bitvec1: & MyBitVec, num_bitvec2: &MyBitVec) -> MyBitVec {
    let sign_true1 = bitvec_tosigntrueform(num_bitvec1);
    let sign_true2 = bitvec_tosigntrueform(num_bitvec2);
    let signtrueform_result = signtrueform_mult(
        &sign_true1,
        &sign_true2,
    );
    let result = signtrueform_tobitvec(&signtrueform_result);
    result
}
//SignTrueform(符号和源码)格式的补码乘法
pub fn signtrueform_mult(signtrueform1: & SignTrueform, signtrueform2: &SignTrueform)->SignTrueform{
    let product_abs = ((signtrueform1.trueform as u128) * (signtrueform2.trueform as u128)) % CONST_P;
    let sign = signtrueform1.sign ^ signtrueform2.sign;
    let result = SignTrueform::new(sign, product_abs as BitStorage);
    result
}

pub fn bitvec_opposite(x_bitvec: &MyBitVec) -> MyBitVec {
    let mut x_signtrueform = bitvec_tosigntrueform(x_bitvec);
    x_signtrueform.sign = !x_signtrueform.sign;
    let out = signtrueform_tobitvec(&x_signtrueform);
    out
}

pub fn signtrueform_opposite(x_signtrueform: &SignTrueform) -> SignTrueform {
    let out = SignTrueform::new(!x_signtrueform.sign, x_signtrueform.trueform);
    out
}

pub fn bitvec_sub(lhs: &MyBitVec, rhs: &MyBitVec) -> MyBitVec {
    let rhs_opp = bitvec_opposite(rhs);
    let out = bitvec_add(&lhs, &rhs_opp);
    out
}


fn booth_multiplication(a: &MyBitVec, b: &MyBitVec) -> MyBitVec {
    let (mut a, mut b) = (a.clone(), b.clone());
    a.reverse();
    b.reverse();
    
    let mut result = std::iter::repeat(false)
        .take(a.len() + b.len())
        .collect::<MyBitVec>();

    let mut carry = false;
    let mut index: usize = 0;

    while index < b.len() - 1 {
        if b[index] && !b[index + 1] {
            add_subtract(&mut result, &a, index, true);
            carry = true;
        } else if !b[index] && b[index + 1] {
            add_subtract(&mut result, &a, index, false);
            carry = true;
        }

        index += 2;
    }

    if carry {
        add_subtract(&mut result, &a, 0, true);
    }

    trim_leading_zeros(&mut result);
    result
}

// Helper function to add or subtract a from result starting at position pos.
fn add_subtract(result: &mut MyBitVec, a: &MyBitVec, pos: usize, add: bool) {
    let mut carry = false;
    let len = std::cmp::min(result.len(), pos + a.len());

    for i in (pos..len).rev() {
        let sum = [result[i], a[i - pos], carry].iter().fold(0, |acc, &x| acc + x as isize) + if add { 1 } else { -1 };
        result.set(i, sum % 2 == 1);
        carry = sum >= 2;
    }
}

// Helper function to remove leading zeros from the result vector.
fn trim_leading_zeros(result: &mut MyBitVec) {
    while result.len() > 1 && result.first().unwrap() == false {
        result.remove(0);
    }
}


#[test]
fn test_booth_mult(){
    use crate::utils::{gen_random_u64_complement_vec}; 

    let sign_trueformx1 = gen_random_u64_complement_vec(4, false);
    let mut num1_vec = signtrueform_tobitvec(&sign_trueformx1);
    println!("plaintext: \n {:?} \n {:?}", sign_trueformx1,num1_vec);
    let sign_trueformx2 = gen_random_u64_complement_vec(4, false);
    let mut num2_vec = signtrueform_tobitvec(&sign_trueformx2);
    println!("plaintext: \n {:?} \n {:?}", sign_trueformx2,num2_vec);

    let result = booth_multiplication(&num1_vec,&num2_vec);
    println!("resukt: \n {:?} \n {:?}", result,bitvec_tosigntrueform(&result));
    let mut result = std::iter::repeat(false)
        .take(num1_vec.len() + num2_vec.len())
        .collect::<MyBitVec>();

    println!("add result: \n {:?} \n {:?}", result, bitvec_tosigntrueform(&result));


}













/*
pub fn bitvec_sub(lhs: &mut BitVec, rhs: &BitVec) {
    let mut borrow = false;
    for (mut l, r) in lhs.iter_mut().rev()
    .zip(rhs.iter().by_vals().rev().chain(core::iter::repeat(false)))
    {
        if borrow {
            *l = !(*l);
            borrow = *l;
        }
        *l = match (*l, r) {
            (true, false) => true,
            (false, true) => {
                borrow = true;
                true
            }
            (_, _) => false,
        };
    }
}


#[test]
fn test_opposite(){
    use crate::utils::{gen_random_u64_complement_vec,vec_tou64,signtrueform_to_complement}; 
    let sign_trueformx1 = gen_random_u64_complement_vec(8, true);
    println!("plaintext: \n {:?}", sign_trueformx1);
    let num1_vec = signtrueform_tobitvec(&sign_trueformx1);
    //println!("vec: \n {:?}", num1_vec);

    let sign_trueformx2 = gen_random_u64_complement_vec(8, true);
    println!("plaintext: \n {:?}", sign_trueformx2);
    let num2_vec = signtrueform_tobitvec(&sign_trueformx2);
    //println!("vec: \n {:?}", num2_vec);

    let signtrueformopposite = signtrueform_opposite(&sign_trueformx1);
    let num1_vecopposite = bitvec_opposite(&num1_vec);
    //println!("opp plaintext: \n {:?}", signtrueformopposite);
    //println!("opp vec: \n {:?}", num1_vecopposite);
    //println!("opp stf: \n {:?}", bitvec_tosigntrueform(&num1_vecopposite));


    let result = bitvec_add(& num1_vecopposite, &num2_vec);

    println!("add result: \n {:?} \n {:?}", result, bitvec_tosigntrueform(&result));


}
#[test]
fn test_mult(){
    use crate::utils::{gen_random_u64_complement_vec,bitvec_tosigntrueform}; 
    let sign_trueformx1 = gen_random_u64_complement_vec(8, true);
    println!("plaintext: \n {:?}", sign_trueformx1);
    let num1_vec = signtrueform_tobitvec(&sign_trueformx1);
    println!("vec: \n {:?}", num1_vec);

    let sign_trueformx2 = gen_random_u64_complement_vec(8, true);
    println!("plaintext: \n {:?}", sign_trueformx2);
    let num2_vec = signtrueform_tobitvec(&sign_trueformx2);
    println!("vec: \n {:?}", num2_vec);
    let result = bitvec_mult(&num1_vec, &num2_vec);
    println!("result: \n {:?}", bitvec_tosigntrueform(&result))
}


#[test]
fn test2(){
    use crate::utils::{gen_random_u64_complement_vec,vec_tou64,signtrueform_to_complement}; 
    let sign_trueformx1 = gen_random_u64_complement_vec(8, false);
    println!("plaintext: \n {:?}", signtrueform_to_complement(&sign_trueformx1));
    let num1_vec = signtrueform_tobitvec(&sign_trueformx1);
    println!("vec: \n {:?}", num1_vec);
    
    let sign_trueformx2 = gen_random_u64_complement_vec(8, true);
    println!("plaintext: \n {:?}", signtrueform_to_complement(&sign_trueformx2));
    let num2_vec = signtrueform_tobitvec(&sign_trueformx2);
    println!("vec: \n {:?}", num2_vec);

    //减法测试
    let mut num22 = num2_vec.clone();
    bitvec_sub(&mut num22, &num1_vec);
    let result = num22.into_vec();
    println!("sub result: \n {:?}", vec_tou64(&result));
    //加法测试
    let mut num22 = num2_vec.clone();
    bitvec_add(&mut num22, &num1_vec);
    let result = num22.into_vec();
    println!("add result: \n {:?}", vec_tou64(&result));

    let result = bitvec_mult(&num1_vec, &num2_vec);
    println!("multi result: \n {:?}", result);
}*/