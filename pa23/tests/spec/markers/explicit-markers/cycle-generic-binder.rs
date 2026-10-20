struct W<T>(T);impl<T> Unpin for W<T> where for<'a> W<T>:Unpin{}fn need<T:Unpin>(){}fn f(){need::<W<u8>>();}
fn main(){}
