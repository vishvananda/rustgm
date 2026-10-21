struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8={let mut n=0;let mut i=0;while i<N{n+=2;i+=1;}n+1};}pub fn run()->bool{S::<2>::C==5&&S::<3>::C==7}
