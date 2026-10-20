fn need<T:Send>(){}fn f<T:Sync>(){need::<&T>();}
fn main(){}
