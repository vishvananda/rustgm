#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_llvm_intrinsics)]
unsafe extern "C"{#[link_name="llvm.ctpop.i32"]safe fn pop(x:u32)->u32;}
pub fn answer()->u32{pop(7)}
