use crate::share::{BitStorage, BitVec,CONST_P};
use crate::utils::{bitvec_tosigntrueform,SignTrueform,signtrueform_tobitvec};
//use std::ops::{Add, Mul};
//bitvec格式的补码加法
pub fn bitvec_add(lhs: & BitVec, rhs: &BitVec) -> BitVec {
    fn bitvec_add_self(lhs: &mut BitVec, rhs: &BitVec) {
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
pub fn bitvec_mult(num_bitvec1: & BitVec, num_bitvec2: &BitVec) -> BitVec {
    let sign_true1 = bitvec_tosigntrueform(num_bitvec1);
    let sign_true2 = bitvec_tosigntrueform(num_bitvec2);
    let signtrueform_result = complement_mult(
        &sign_true1,
        &sign_true2,
    );
    let result = signtrueform_tobitvec(&signtrueform_result);
    result
}
//SignTrueform(符号和源码)格式的补码乘法
pub fn complement_mult(signtrueform1: & SignTrueform, signtrueform2: &SignTrueform)->SignTrueform{
    let product_abs = ((signtrueform1.trueform as u128) * (signtrueform2.trueform as u128)) % CONST_P;
    let sign = signtrueform1.sign ^ signtrueform2.sign;
    let result = SignTrueform::new(sign, product_abs as BitStorage);
    result
}


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
}