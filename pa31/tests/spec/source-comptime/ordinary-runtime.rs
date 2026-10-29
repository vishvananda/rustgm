#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#[rustc_comptime]fn id(x:u32)->u32{x}
pub fn observed()->u32{id(7)}
