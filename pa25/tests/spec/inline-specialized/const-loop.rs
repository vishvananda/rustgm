const fn f<const N:u8>()->u8{const{let mut n=0;while n<N{n+=1;}n}}pub fn run()->bool{f::<2>()==2&&f::<3>()==3}
