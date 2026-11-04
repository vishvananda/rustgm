#![feature(unsized_fn_params)]
fn take<T:?Sized>(x:T){let _=&x;}fn main(){}
