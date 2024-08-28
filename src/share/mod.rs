use bitvec::order::Msb0;
use bitvec;
use std::fmt::Debug;
use std::hash::Hash;
pub mod xor_sharing;
pub mod add_sharing;
//pub(crate) mod util;

//定义Bitvec数据类型，MSB
pub type BitStorage = u64;
pub const CONST_P_64: u128 = 2u128.pow(64);
pub type MyBitSlice<T = BitStorage> = bitvec::slice::BitSlice<T,Msb0>;
pub type MyBitVec<T = BitStorage> = bitvec::vec::BitVec<T,Msb0>;

//从Flute抄几个trait
pub trait Gate: Clone + Hash + Ord + PartialEq + Eq + Send + Sync + Debug + 'static {
    type Share: Copy + Clone + Default + Debug;

    fn is_interactive(&self) -> bool;

    fn input_size(&self) -> usize;

//暂时用不到这个    fn as_base_gate(&self) -> Option<&BaseGate<Self::Share>>;

    fn evaluate_non_interactive(
        &self,
        party_id: usize,
        inputs: impl IntoIterator<Item = Self::Share>,
    ) -> Self::Share;

    fn is_non_interactive(&self) -> bool {
        !self.is_interactive()
    }
}

pub trait ShareStorage<Share>:
    IntoIterator<Item = Share> + FromIterator<Share> + Clone + Default + Debug
{
    fn len(&self) -> usize;
    fn repeat(val: Share, len: usize) -> Self;
    fn set(&mut self, idx: usize, val: Share);
    fn get(&self, idx: usize) -> Share;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl ShareStorage<bool> for MyBitVec {
    fn len(&self) -> usize {
        MyBitVec::len(self)
    }

    fn repeat(val: bool, len: usize) -> Self {
        MyBitVec::repeat(val, len)
    }

    fn set(&mut self, idx: usize, val: bool) {
        MyBitSlice::set(self, idx, val)
    }

    fn get(&self, idx: usize) -> bool {
        self[idx]
    }
}
pub trait Sharing {
    type Plain: Copy + Clone + Default + Debug;
    type ShareType: ShareStorage<Self::Plain>;
    fn share(&mut self, input: Self::ShareType) -> [Self::ShareType; 2];

    fn reconstruct(&mut self, shares: [Self::ShareType; 2]) -> Self::ShareType;
}





#[test]
fn test_bit_vector() { //每个bit都存成u8的vec
    let plaintext: Vec<u8> =vec![0,1,1,0,1,1,0,1];
    //输入1时，MSB会输出0 0 0 0 0 0 0 1， LSB会输出1 0 0 0 0 0 0 0
    let mut plain_vex = MyBitVec::from_vec(plaintext);
    print!("{}, {}", plain_vex, plain_vex.len()); //长度为64
    plain_vex.push(true);
    print!("{}, {}", plain_vex, plain_vex.len());

}

/*
#[test]
fn test2() { //测试每个bit只存成1bit长的vec
    use bitvec::prelude::bitvec;
    let mut bits = bitvec![0, 1, 0, 0, 1];
    println!("{:?}", bits);
    bits.push(true);
    println!("{:?}", bits);
    let mut bits2 = bitvec![0, 0, 0, 0, 0,1];
}
 */