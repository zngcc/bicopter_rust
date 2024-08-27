use crate::share::{BitStorage, MyBitVec,CONST_P};
use crate::utils::{SignTrueform,signtrue_tobitvec};
//use std::ops::{Add, Mul};

pub fn bitvec_add(a: &MyBitVec, b: &MyBitVec) -> MyBitVec {
    let (mut a,mut b) = (a.clone(), b.clone());
    let mut carry = false;
    let mut result  = std::iter::repeat(false)
    .take(a.len())
    .collect::<MyBitVec>();
    if a.len() > b.len() {
        let mut buwei = std::iter::repeat(b[0])
        .take(a.len() - b.len())
        .collect::<MyBitVec>();
        buwei.extend(b.iter());
        b = buwei;
    } else if a.len() < b.len() {
        let mut buwei = std::iter::repeat(a[0])
        .take(b.len() - a.len())
        .collect::<MyBitVec>();
        buwei.extend(a.iter());
        a = buwei;
    }
    for i in (0..a.len()).rev() {
        let sum = a[i] ^ b[i] ^ carry;
        result.set(i, sum);
        // 更新进位
        carry = a[i] & b[i] | a[i] & carry | b[i] & carry;
    }
    result
}
//bitvec格式的补码乘法




//SignTrueform(符号和源码)格式的补码乘法
pub fn signtrueform_mult(signtrueform1: & SignTrueform, signtrueform2: &SignTrueform)->SignTrueform{
    let product_abs = ((signtrueform1.trueform as u128) * (signtrueform2.trueform as u128)) % CONST_P;
    let sign = signtrueform1.sign ^ signtrueform2.sign;
    let result = SignTrueform::new(sign, product_abs as BitStorage);
    result
}

pub fn bitvec_opposite(x_bitvec: &MyBitVec) -> MyBitVec {
    let mut result = x_bitvec.clone();
    for i in 0..result.len() {
        let opp = !result[i];
        result.set(i, opp);
    }
    result = bitvec_add(&result, &signtrue_tobitvec(false, 1));
    result
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


pub fn bitvec_mult(num1: &MyBitVec, num2: &MyBitVec) -> MyBitVec {
    let mut m = num1.clone();
    let mut q = num2.clone();
    q.push(false); // 添加额外的一位
    m.insert(0, m[0]);

    let mut r = std::iter::repeat(false)
    .take(num1.len()+1)
    .collect::<MyBitVec>();

    let mopp = bitvec_opposite(&m);
    //println!("-m,m,q:\n{:?}\n{:?}\n{:?}\n",mopp,m,q);
    for _i in 0..q.len()-1 {
        // 根据Booth算法的规则更新r
        //println!("{:?} {:?} {:?} \n", i, q[q.len()-2], q[q.len()-1]);
        match (q[q.len()-2], q[q.len()-1]) {
            (true, false) => {
                //println!("r+:\n{:?}\n{:?}\n", r,mopp);
                r = bitvec_add(&r,&mopp);
                //println!("r:\n{:?}", r);
            }, // 如果是10，则减去m
            (false, true) => {
                //println!("r+:\n{:?}\n{:?}\n", r,m);
                r = bitvec_add(&r,&m);
                //println!("r:\n{:?}", r);
            }, // 如果是01，则加上m
            _ => (),                  // 否则不做操作
        }
        // 右移
        let new_q0 = r.pop().unwrap_or(false);
        q.insert(0, new_q0);
        r.insert(0, r[0]);
        q.pop();
    }


    // 最后一次循环之后的结果需要加上m或减去m
    //match (q[q.len()-2], q[q.len()-1]) {
    //    (true, false) => r = bitvec_sub(&r,&m), // 如果是10，则减去m
    //    (false, true) => r = bitvec_add(&r,&m), // 如果是01，则加上m
    //    _ => (),                  // 否则不做操作
    //}

    q.pop();
    q
}


#[test]
fn test_mult(){

    use crate::utils::{gen_random_u64_complement_vec, signtrueform_tobitvec, bitvec_tosigntrueform}; 
    let sign_trueformx1 = gen_random_u64_complement_vec(32, true);
    let num1_vec = signtrueform_tobitvec(&sign_trueformx1);
    println!("num1: \n {:?} \n {:?}\n", sign_trueformx1,num1_vec);
    let sign_trueformx2 = gen_random_u64_complement_vec(8, false);
    let num2_vec = signtrueform_tobitvec(&sign_trueformx2);
    println!("num2: \n {:?} \n {:?}\n", sign_trueformx2,num2_vec);
    let result = bitvec_mult(& num1_vec,&num2_vec);

    println!("add:\n{:?}", (&result, bitvec_tosigntrueform(&result)));
    

}







/*
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
#[test]
fn test_booth_mult(){
    use crate::utils::{gen_random_u64_complement_vec}; 
    let sign_trueformx1 = gen_random_u64_complement_vec(32, false);
    let mut num1_vec = signtrueform_tobitvec(&sign_trueformx1);
    //let mut num1_vec =signtrue_tobitvec(false, 32); 
    println!("num1: \n {:?} \n", (sign_trueformx1,&num1_vec));
    let sign_trueformx2 = gen_random_u64_complement_vec(32, false);
    let mut num2_vec = signtrueform_tobitvec(&sign_trueformx2);
    //let mut num2_vec =signtrue_tobitvec(false, 32); 
    println!("num2: \n {:?} \n ", (sign_trueformx2,&num2_vec));
    //let result = add_complement(& num1_vec,&num2_vec);
    //println!("add:\n{:?}", result);
    

    let (mut a, mut b) = (num1_vec.clone(), num2_vec.clone());

    //let mut result = std::iter::repeat(false)
    //    .take(a.len()+b.len())
    //    .collect::<MyBitVec>();
    let mut m = a.clone();
    let mut q = b.clone();
    q.push(false); // 添加额外的一位
    m.insert(0, m[0]);

    let mut r = std::iter::repeat(false)
    .take(a.len()+1)
    .collect::<MyBitVec>();
    // 扩展乘数
    
    //println!("m,q:\n{:?}\n{:?}\n", m,q);
    let mopp = bitvec_opposite(&m);
    println!("-m,m,q:\n{:?}\n{:?}\n{:?}\n",mopp,m,q);
    for i in 0..q.len()-1 {
        // 根据Booth算法的规则更新r
        //println!("{:?} {:?} {:?} \n", i, q[q.len()-2], q[q.len()-1]);
        match (q[q.len()-2], q[q.len()-1]) {
            (true, false) => {
                //println!("r+:\n{:?}\n{:?}\n", r,mopp);
                r = bitvec_add(&r,&mopp);
                //println!("r:\n{:?}", r);
            }, // 如果是10，则减去m
            (false, true) => {
                //println!("r+:\n{:?}\n{:?}\n", r,m);
                r = bitvec_add(&r,&m);
                //println!("r:\n{:?}", r);
            }, // 如果是01，则加上m
            _ => (),                  // 否则不做操作
        }
        //println!("q,r:\n{:?}\n{:?}\n", q,r);
        // 循环右移
        let new_q0 = r.pop().unwrap_or(false);
        q.insert(0, new_q0);
        r.insert(0, r[0]);
        q.pop();
        //println!("q,r2:\n{:?}\n{:?}\n", q,r);
    }


    // 最后一次循环之后的结果需要加上m或减去m
    //match (q[q.len()-2], q[q.len()-1]) {
    //    (true, false) => r = bitvec_sub(&r,&m), // 如果是10，则减去m
    //    (false, true) => r = bitvec_add(&r,&m), // 如果是01，则加上m
    //    _ => (),                  // 否则不做操作
    //}

    q.pop();
    println!("r q \n{:?}\n{:?}", r, q);
    //println!("r,m \n{:?}\n{:?}\n", r, m);
    // 将结果复制到最终结果向量中
    //println!("l \n{:?}\n", (result[..a.len()].len(), result[a.len()..].len(), r[(r.len()-a.len())..].len(), q[0..a.len()].len()));
    //result[..a.len()].copy_from_bitslice(&r[(r.len()-a.len())..]);
    //result[a.len()..].copy_from_bitslice(&q[0..a.len()]);

    println!("r\n{:?}\n{:?}", q, bitvec_tosigntrueform(&q));

}





#[test]
fn test_add(){
    use crate::utils::{gen_random_u64_complement_vec}; 
    let sign_trueformx1 = gen_random_u64_complement_vec(60, true);
    let mut num1_vec = signtrueform_tobitvec(&sign_trueformx1);
    println!("num1: \n {:?} \n {:?}\n", sign_trueformx1,num1_vec);
    let sign_trueformx2 = gen_random_u64_complement_vec(8, false);
    let mut num2_vec = signtrueform_tobitvec(&sign_trueformx2);
    println!("num2: \n {:?} \n {:?}\n", sign_trueformx2,num2_vec);
    let result = bitvec_add(& num1_vec,&num2_vec);

    println!("add:\n{:?}", result);
    

}



pub fn bitvec_opposite0(x_bitvec: &MyBitVec) -> MyBitVec {
    let mut x_signtrueform = bitvec_tosigntrueform(x_bitvec);
    x_signtrueform.sign = !x_signtrueform.sign;
    let out = signtrueform_tobitvec(&x_signtrueform);
    out
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
fn opp(){
    use crate::utils::{gen_random_u64_complement_vec}; 
    let sign_trueformx1 = gen_random_u64_complement_vec(4, false);
    let mut num1_vec = signtrueform_tobitvec(&sign_trueformx1);
    
    let mut result = num1_vec.clone();
    result.insert(0, false);
    println!("{:?}", result);
    for i in 0..result.len() {
        let opp = !result[i];
        result.set(i, opp);
    }
    println!("{:?}", result);
    result = bitvec_add(&result, &signtrue_tobitvec(false, 1));
    println!("{:?}", result);

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

//bitvec格式的补码加法
/*
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
*/