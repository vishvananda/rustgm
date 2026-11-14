#![feature(const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
pub const fn apply<F:[const]FnOnce(u8)->u8>(f:&mut F)->u8{f(41)}
fn main(){}
