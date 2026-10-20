use core::marker as m;fn need<T:m::Send>(){}fn f(){need::<u8>();}
fn main(){}
