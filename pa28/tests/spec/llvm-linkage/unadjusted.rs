#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(abi_unadjusted,link_llvm_intrinsics)]unsafe extern "unadjusted"{#[link_name="llvm.ctpop.i32"]fn operation(x:u32)->u32;}pub fn answer(x:u32)->u32{unsafe{operation(x)}}
