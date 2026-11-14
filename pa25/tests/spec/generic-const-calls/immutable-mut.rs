#![feature(const_trait_impl,const_destruct,const_cmp)]
use core::marker::Destruct;
pub const fn apply<F:[const]FnMut(u8)->u8+[const]Destruct>(f:F,x:u8)->u8{f(x)}
fn main(){}
