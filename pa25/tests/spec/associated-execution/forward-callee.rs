struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8={let n=f(N);n+1};}const fn f(n:u8)->u8{n*2}pub fn run()->bool{S::<2>::C==5&&S::<3>::C==7}
