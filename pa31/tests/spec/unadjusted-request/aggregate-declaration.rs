#![allow(internal_features,unused_features,dead_code,unexpected_cfgs)]
#![feature(abi_unadjusted)]
#[repr(C)]pub struct Pair{pub a:u64,pub b:u64}
pub extern "unadjusted" fn active(value:Pair)->Pair{value}
