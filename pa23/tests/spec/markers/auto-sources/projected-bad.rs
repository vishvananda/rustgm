trait A{type V;}struct W<T:A>(T::V);fn need<T:Send>(){}fn f<T:A>(){need::<W<T>>();}
fn main(){}
