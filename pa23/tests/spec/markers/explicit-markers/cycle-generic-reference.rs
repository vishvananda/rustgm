struct W<T>(T);unsafe impl<T> Sync for W<T> where for<'a> &'a W<T>:Sync{}fn need<T:Sync>(){}fn f(){need::<W<*const u8>>();}
fn main(){}
