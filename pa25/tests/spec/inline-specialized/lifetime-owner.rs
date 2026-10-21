fn f<'a>(_:&'a u8)->u8{const{7}}pub fn run()->bool{let x=1;f(&x)==7&&f(&2)==7}
