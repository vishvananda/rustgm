trait A{}struct W<T>(T);impl<T:A> Unpin for W<T>{}fn need<T:Unpin>(){}fn f<T>() where W<T>:Unpin{need::<W<T>>();}
fn main(){}
