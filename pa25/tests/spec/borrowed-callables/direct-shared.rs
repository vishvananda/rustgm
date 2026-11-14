#![feature(const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
pub const fn apply<F:[const]Fn(u8)->u8>(f:&F)->u8{f(41)}
fn main(){}
