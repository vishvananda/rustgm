struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8={let n=const{N+1};n+const{const{1}}};}pub fn run()->bool{S::<2>::C==4&&S::<3>::C==5}
