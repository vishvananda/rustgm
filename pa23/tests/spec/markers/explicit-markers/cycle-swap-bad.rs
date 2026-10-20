struct W<'a,'b>(&'a u8,&'b u8);impl<'a,'b> Unpin for W<'a,'b> where 'a:'b,W<'b,'a>:Unpin{}fn need<T:Unpin>(){}fn f<'a,'b>(){need::<W<'a,'b>>();}
fn main(){}
