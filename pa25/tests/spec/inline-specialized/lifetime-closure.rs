fn f<'a>(_:&'a u8)->u8{let c=||const{7};c()}pub fn run()->bool{let x=1;f(&x)==7&&f(&2)==7}
