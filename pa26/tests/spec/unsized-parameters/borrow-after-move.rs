#![feature(unsized_fn_params)]
fn take(_:str){}fn forward(x:str){take(x);let _=&x;}fn main(){}
