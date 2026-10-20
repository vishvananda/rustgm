fn f<'a>(x:&'a u8)->impl Copy+'a{x}fn take<T>(_:T){}fn main(){let mut a=1;let x=f(&a);a=2;take(x);}
