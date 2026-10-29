#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#[rustc_comptime]fn id(x:u32)->u32{x}
const VALUE:u32=id(7);
#[unsafe(no_mangle)]pub extern "C" fn observed()->u32{VALUE}
