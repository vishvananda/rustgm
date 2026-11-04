#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(allow_internal_unstable)]
#[allow_internal_unstable(link_llvm_intrinsics)]macro_rules! make{($i:item)=>{$i};}make!{unsafe extern "C"{#[link_name="llvm.ctpop.i32"]fn pop(x:u32)->u32;}
}pub fn answer()->u8{7}
