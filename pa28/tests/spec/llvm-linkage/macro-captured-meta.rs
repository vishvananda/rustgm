#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(allow_internal_unstable)]
#[allow_internal_unstable(link_llvm_intrinsics)]macro_rules! make{($a:meta)=>{unsafe extern "C"{#[$a]fn pop(x:u32)->u32;}};}make!{link_name="llvm.ctpop.i32"}pub fn answer()->u32{unsafe{pop(7)}}
