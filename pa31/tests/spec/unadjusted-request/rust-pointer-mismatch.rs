#![allow(internal_features,unused_features,dead_code,unexpected_cfgs)]
#![feature(abi_unadjusted)]
pub extern "unadjusted" fn active()->u8{7}
pub const BAD:fn()->u8=active;
