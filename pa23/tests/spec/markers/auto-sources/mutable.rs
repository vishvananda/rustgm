fn need<T:Send>(){}fn f<T:Send>(){need::<&mut T>();}
fn main(){}
