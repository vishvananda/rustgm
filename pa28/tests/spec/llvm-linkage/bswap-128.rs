#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_llvm_intrinsics)]
unsafe extern "C"{#[link_name="llvm.bswap.i128"]fn operation(x:u128)->u128;}pub fn answer(x:u128)->u128{unsafe{operation(x)}}
