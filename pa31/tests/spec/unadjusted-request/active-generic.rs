#![allow(internal_features,unused_features,dead_code,unexpected_cfgs)]
#![feature(abi_unadjusted)]
pub extern "unadjusted" fn identity<T>(value:T)->T{value}
