fn bad()->u8{7}struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8={if false{bad()}else{N}};}pub fn run()->bool{S::<2>::C==2}
fn main(){let _=run();}