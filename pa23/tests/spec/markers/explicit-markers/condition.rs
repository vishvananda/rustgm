trait A{}impl A for u8{}struct W<T>(T);impl<T:A> Unpin for W<T>{}fn need<T:Unpin>(){}fn f(){need::<W<u8>>();}
fn main(){}
