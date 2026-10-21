fn f<const N:u8>()->u8{let c=||const{N+1};c()}pub fn run()->bool{f::<2>()==3&&f::<3>()==4}
