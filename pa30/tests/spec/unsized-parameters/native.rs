#![feature(unsized_fn_params)]
unsafe extern "C" {fn observe(value:u32);}
struct Value;
impl Drop for Value {fn drop(&mut self){unsafe{observe(9);}}}
fn consume<T:?Sized>(x:T){let _=&x;unsafe{observe(7);}}
fn main(){consume(Value);}
