fn f<'a,'b>(_:&'a u8,_:&'b u8)->impl Copy+'a+'static+'b{7u8}fn take<T:'static>(_:T){}fn main(){let x;{let a=1;let b=2;x=f(&a,&b);}take(x);}
