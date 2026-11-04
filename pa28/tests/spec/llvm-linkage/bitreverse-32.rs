#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_llvm_intrinsics)]
unsafe extern "C"{#[link_name="llvm.bitreverse.i32"]fn operation(x:u32)->u32;}pub fn answer(x:u32)->u32{unsafe{operation(x)}}
