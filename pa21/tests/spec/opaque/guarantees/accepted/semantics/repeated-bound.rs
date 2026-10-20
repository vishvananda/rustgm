fn f<'a,'b>(x:&'a u8,_:&'b u8)->impl Copy+'a+'a{x}fn take<T>(_:T){}fn main(){let a=1;let x;{let b=2;x=f(&a,&b);}take(x);}
