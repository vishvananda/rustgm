fn need<T:Copy>(){}fn f<T:Send>(){need::<T>();}
fn main(){}
