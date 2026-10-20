trait Send{}fn need<T:Send>(){}fn f(){need::<u8>();}
fn main(){}
