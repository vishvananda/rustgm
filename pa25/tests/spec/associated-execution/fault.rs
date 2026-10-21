struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8={let n=N;1/n};}pub fn run()->bool{S::<0>::C==0}
