trait All<'a,'b>:'b{}impl<'a,'b>All<'a,'b> for &'b u8{}fn f<'b,'c>(x:&'b u8,_:&'c u8)->impl for<'a>All<'a,'b>{x}fn take<T>(_:T){}fn main(){let a=1;let x;{let b=2;x=f(&a,&b);}take(x);}
