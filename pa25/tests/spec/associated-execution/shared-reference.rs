struct S<const N:u8>;impl<const N:u8>S<N>{const C:&'static u8={let value=&N;value};}pub fn run()->bool{*S::<2>::C==2&&*S::<3>::C==3}
