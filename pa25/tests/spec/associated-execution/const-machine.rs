struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8={let n=N;n+1};}const fn f<const N:u8>()->u8{S::<N>::C}const A:u8=f::<2>();const B:u8=f::<3>();pub fn run()->bool{A==3&&B==4}
