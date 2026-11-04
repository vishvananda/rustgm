#![feature(unsized_fn_params)]
fn take<T:?Sized>(_x:T){}fn main(){take(7u32);}
