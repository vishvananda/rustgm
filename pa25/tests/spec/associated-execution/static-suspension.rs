struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8={let n=BASE;n+N};}static BASE:u8=7;pub fn run()->bool{S::<2>::C==9&&S::<3>::C==10}
