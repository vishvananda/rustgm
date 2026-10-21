fn f<const N:u8>(x:u8)->u8{let c=||x+const{N};c()}pub fn run()->bool{f::<2>(5)==7&&f::<3>(5)==8}
