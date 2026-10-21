const fn f<const N:u8>()->u8{const{f::<N>()}}const A:u8=f::<2>();pub fn run()->bool{A==7}
