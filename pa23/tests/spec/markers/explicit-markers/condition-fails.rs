trait A{}struct W<T>(T);impl<T:A> Unpin for W<T>{}fn need<T:Unpin>(){}fn f(){need::<W<u8>>();}
fn main(){}
