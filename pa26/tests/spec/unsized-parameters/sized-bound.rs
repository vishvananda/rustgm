#![feature(unsized_fn_params)]
fn sized<T>(_:T){}fn take(x:str){sized(x);}fn main(){}
