trait A<T:Send>{}fn need<T:?Sized+Send>(){}fn f(){need::<dyn A<u8>>();}
fn main(){}
