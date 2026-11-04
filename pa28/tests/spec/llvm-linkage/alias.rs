#![allow(internal_features,dead_code)]
#![feature(link_llvm_intrinsics)]
type Word=u32;unsafe extern "C"{#[link_name="llvm.ctpop.i32"]fn operation(x:Word)->u32;}pub fn answer(x:Word)->u32{unsafe{operation(x)}}
