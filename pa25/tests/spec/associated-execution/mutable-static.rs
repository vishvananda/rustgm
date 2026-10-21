static mut X:u8=7;struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8={let n=unsafe{X};n+N};}pub fn run()->bool{S::<2>::C==9}
