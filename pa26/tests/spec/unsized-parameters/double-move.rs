#![feature(unsized_fn_params)]
fn take(_:str){}fn forward(x:str){take(x);take(x);}fn main(){}
