#![feature(unsized_fn_params)]
fn take(x:str){let f=||{let _=&x;};f();}fn main(){}
