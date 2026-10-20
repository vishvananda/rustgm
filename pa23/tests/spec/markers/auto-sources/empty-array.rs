fn need<T:Send>(){}fn f(){need::<[*const u8;0]>();}
fn main(){}
