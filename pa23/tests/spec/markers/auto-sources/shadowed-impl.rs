trait Send{}impl Send for u8{}fn need<T:Send>(){}fn f(){need::<u8>();}
fn main(){}
