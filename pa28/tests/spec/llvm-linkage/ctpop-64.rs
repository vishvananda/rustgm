#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_llvm_intrinsics)]
unsafe extern "C"{#[link_name="llvm.ctpop.i64"]fn operation(x:u64)->u64;}pub fn answer(x:u64)->u64{unsafe{operation(x)}}
