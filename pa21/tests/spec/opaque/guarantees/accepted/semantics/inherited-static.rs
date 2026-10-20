trait Forever:'static{}impl Forever for u8{}fn f<'a>(_:&'a u8)->impl Forever{7u8}fn take<T:'static>(_:T){}fn main(){let x;{let a=1;x=f(&a);}take(x);}
