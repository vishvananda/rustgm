#![feature(unsized_fn_params)]
fn take(x:str){let f=move||{let _=&x;};f();}fn main(){}
