struct W<T>(T);impl<T:Sync> Unpin for W<T>{}fn need<T:Unpin>(){}fn f<T:Sync>(){need::<W<T>>();}
fn main(){}
