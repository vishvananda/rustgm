#![feature(const_closures,const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
pub const fn check<F:[const]Fn(u8)->u8+[const]Destruct>(f:F){let _c=const move |a| f(a);}
fn main(){}
