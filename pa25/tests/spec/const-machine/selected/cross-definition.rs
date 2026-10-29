struct S<const N:u8>;impl<const N:u8>S<N>{const BASE:u8={let n=N;n+7};const C:u8={let n=S::<0>::BASE;n+N};}pub fn run()->bool{S::<2>::C==9&&S::<3>::C==10}
fn main(){let _=run();}