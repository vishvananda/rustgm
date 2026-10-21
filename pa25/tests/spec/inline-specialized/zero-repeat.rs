const fn bad(n:u8)->u8{1/n}fn f<const N:usize>()->[u8;N]{[const{bad(0)};N]}pub fn run()->bool{let _:[u8;0]=f::<0>();true}
