fn f<const N:usize>()->[u8;N]{const{[7;N]}}pub fn run()->bool{let a=f::<2>();let b=f::<3>();a[1]==7&&b[2]==7}
