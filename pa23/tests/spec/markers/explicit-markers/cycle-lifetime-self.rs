struct W<'a>(&'a u8);impl<'a> Unpin for W<'a> where W<'a>:Unpin{}fn need<T:Unpin>(){}fn f<'a>(){need::<W<'a>>();}
fn main(){}
