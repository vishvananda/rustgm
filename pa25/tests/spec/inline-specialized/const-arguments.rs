const fn f<const N:u8>()->u8{const{N+1}}const A:u8=f::<2>();const B:u8=f::<3>();pub fn run()->bool{A==3&&B==4&&f::<2>()==3}
