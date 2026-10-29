fn bad()->u8{7}struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8={let n=bad();n};}pub fn run()->bool{true}
fn main(){let _=run();}