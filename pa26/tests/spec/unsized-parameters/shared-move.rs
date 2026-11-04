#![feature(unsized_fn_params)]
fn take(_:str){}fn forward(x:&str){take(*x);}fn main(){}
