struct W<T>(T);fn need<T:Send>(){}fn f<T:Send>(){need::<W<T>>();}
fn main(){}
