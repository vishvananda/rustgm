struct W<'a>(&'a u8);impl<'a> Unpin for W<'a> where W<'static>:Unpin{}fn need<T:Unpin>(){}fn f<'a>(){need::<W<'a>>();}
fn main(){}
