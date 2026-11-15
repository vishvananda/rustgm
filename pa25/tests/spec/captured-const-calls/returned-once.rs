#![feature(const_closures,const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
pub const fn wrap<F:[const]FnOnce(u8)->u8+[const]Destruct>(f:F)->impl [const]FnOnce(u8)->u8+[const]Destruct{const move |a| f(a)}
fn main(){}
