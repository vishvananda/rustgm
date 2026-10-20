struct W<T>(T);impl<T:Sync> Unpin for W<T> where W<T>:Unpin{}fn need<T:Unpin>(){}fn f(){need::<W<*const u8>>();}
fn main(){}
