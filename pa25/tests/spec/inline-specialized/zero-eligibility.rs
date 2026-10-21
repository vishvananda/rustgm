fn bad()->u8{7}fn f<const N:usize>()->u8{let _=[const{bad()};N];7}pub fn run()->bool{f::<0>()==7}
