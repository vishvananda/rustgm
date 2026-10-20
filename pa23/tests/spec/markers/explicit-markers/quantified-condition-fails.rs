struct W<T>(T);impl<T> Unpin for W<T> where for<'a> &'a T:Send{}fn need<T:Unpin>(){}fn f<T:Send>(){need::<W<T>>();}
fn main(){}
