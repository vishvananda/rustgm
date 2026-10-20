struct W<T>(T);unsafe impl<T> Send for W<T>{}fn need<T:Send>(){}fn f<T>(){need::<W<T>>();}
fn main(){}
