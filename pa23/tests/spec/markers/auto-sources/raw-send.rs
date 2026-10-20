fn need<T:Send>(){}fn f(){need::<*const u8>();}
fn main(){}
