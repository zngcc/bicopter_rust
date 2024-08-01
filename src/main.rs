use Bicopter::share::BitVec;


fn main() {
    let plaintext: Vec<u8> =vec![0,1,1,0,1,1,0,1];
    let d = BitVec::from_vec(plaintext);
    print!("{}, {}", d, d[0])
}




