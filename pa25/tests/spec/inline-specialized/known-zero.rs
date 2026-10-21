const fn bad(n:u8)->u8{1/n}fn f<const N:usize>()->u8{let _:[u8;0]=[const{bad(N as u8)};0];7}pub fn run()->bool{f::<0>()==7&&f::<1>()==7}
