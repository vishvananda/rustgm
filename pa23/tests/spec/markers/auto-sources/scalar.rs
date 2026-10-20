fn need<T:Send+Sync+Unpin>(){}fn f(){need::<u8>();}
fn main(){}
