#![feature(unsized_fn_params)]
struct Value;
impl Drop for Value {fn drop(&mut self){println!("9");}}
fn consume<T:?Sized>(x:T){let _=&x;println!("7");}
fn main(){consume(Value);}
