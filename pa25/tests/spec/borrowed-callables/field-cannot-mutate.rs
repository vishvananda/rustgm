#![feature(const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
pub struct Borrowed<'a,F>{pub f:&'a mut F}
pub const fn apply<F:[const]FnMut(u8)->u8>(x:&Borrowed<'_,F>)->u8{(x.f)(41)}
fn main(){}
