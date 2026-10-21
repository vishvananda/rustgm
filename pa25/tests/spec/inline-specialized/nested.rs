const fn f<const N:u8>()->u8{const{(const{N+1})+1}}pub fn run()->bool{f::<2>()==4&&f::<3>()==5}
