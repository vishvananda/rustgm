fn f<'a>(_:&'a u8)->impl Copy{7u8}fn take<T:'static>(_:T){}fn main(){let a=1;take(f(&a));}
