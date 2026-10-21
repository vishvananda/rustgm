fn f<const N:usize>()->[u8;N]{[const{7};N]}pub fn run()->bool{let _:[u8;0]=f::<0>();let x=f::<2>();x[1]==7}
