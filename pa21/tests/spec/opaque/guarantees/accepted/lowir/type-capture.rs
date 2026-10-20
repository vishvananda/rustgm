fn f<T>(_:T)->impl Copy+'static{13u8}fn take<T:'static>(_:T){}fn main(){let x;{let a=1;x=f(&a);}take(x);}
