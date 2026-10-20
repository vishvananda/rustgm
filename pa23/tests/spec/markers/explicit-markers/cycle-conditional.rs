struct W<T>(T);impl<T> Unpin for W<T> where W<T>:Unpin{}fn need<T:Unpin>(){}fn f(){need::<W<u8>>();}
fn main(){}
