fn f<'a,'b>(_:&'a u8,_:&'b u8)->impl Copy+'a+'b{7u8}fn take<T>(_:T){}fn main(){let a=1;let b=2;take(f(&a,&b));}
