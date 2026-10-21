struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8={let n=S::<N>::C;n+1};}pub fn run()->bool{S::<2>::C==3}
