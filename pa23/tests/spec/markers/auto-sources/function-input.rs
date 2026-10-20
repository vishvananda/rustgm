struct W<T>(fn(T));fn need<T:Send>(){}fn f<T>(){need::<W<T>>();}
fn main(){}
