struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8={if N==0{7}else{S::<0>::C}};}pub fn run()->bool{S::<2>::C==7&&S::<3>::C==7}
