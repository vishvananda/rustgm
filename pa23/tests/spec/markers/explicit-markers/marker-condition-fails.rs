struct W<T>(T);impl<T:Sync> Unpin for W<T>{}fn need<T:Unpin>(){}fn f(){need::<W<*const u8>>();}
fn main(){}
