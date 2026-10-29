trait T{const C:u8;}struct S<const N:u8>;impl<const N:u8>T for S<N>{const C:u8={let n=N;n+1};}pub fn run()->bool{<S<2> as T>::C==3&&<S<3> as T>::C==4}
fn main(){let _=run();}