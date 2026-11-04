#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_llvm_intrinsics)]
unsafe extern "C"{#[link_name="llvm.bitreverse.i16"]fn operation(x:u16)->u16;}pub fn answer(x:u16)->u16{unsafe{operation(x)}}
