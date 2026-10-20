trait All<'a>:'static{}impl<'a>All<'a> for u8{}fn f<'b>(_:&'b u8)->impl for<'a>All<'a>{7u8}fn take<T:'static>(_:T){}fn main(){let x;{let a=1;x=f(&a);}take(x);}
