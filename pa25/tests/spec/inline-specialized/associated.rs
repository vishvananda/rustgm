struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8=const{N+1};}pub fn run()->bool{S::<2>::C==3&&S::<3>::C==4}
