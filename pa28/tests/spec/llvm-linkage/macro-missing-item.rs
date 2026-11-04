#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(allow_internal_unstable)]
macro_rules! make{()=>{unsafe extern "C"{#[link_name="llvm.ctpop.i32"]fn pop(x:u32)->u32;}
};}make!();pub fn answer()->u32{unsafe{pop(7)}}
