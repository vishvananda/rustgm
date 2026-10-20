fn f<'a>(x:&'a u8)->impl Copy+'a{x}fn take<T>(_:T){}fn main(){let x;{let a=1;x=f(&a);}take(x);}
