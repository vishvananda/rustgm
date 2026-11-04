#![allow(internal_features,unused_features,dead_code,unexpected_cfgs)]
#![feature(abi_unadjusted)]
pub type Callback=extern "unadjusted" fn(u8)->u8;
