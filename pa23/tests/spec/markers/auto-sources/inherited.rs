trait A:Send{}fn need<T:?Sized+Send>(){}fn f(){need::<dyn A>();}
fn main(){}
