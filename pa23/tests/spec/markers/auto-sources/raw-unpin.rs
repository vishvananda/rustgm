fn need<T:Unpin>(){}fn f(){need::<*const u8>();}
fn main(){}
