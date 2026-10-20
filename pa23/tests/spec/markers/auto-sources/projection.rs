trait A{type V:Send;}fn need<T:Send>(){}fn f<T:A>(){need::<T::V>();}
fn main(){}
