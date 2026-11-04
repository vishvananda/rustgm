#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_llvm_intrinsics)]
unsafe extern "C"{#[link_name="llvm.bitreverse.i8"]fn operation(x:u8)->u8;}pub fn answer(x:u8)->u8{unsafe{operation(x)}}
