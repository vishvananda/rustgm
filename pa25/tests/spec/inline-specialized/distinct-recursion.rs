const fn f<const N:u8>()->u8{const{if N==0{7}else{f::<0>()}}}pub fn run()->bool{f::<2>()==7&&f::<3>()==7}
