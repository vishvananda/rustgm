#![allow(internal_features,unused_features,dead_code,unexpected_cfgs)]
#![feature(allow_internal_unstable)]
#[allow_internal_unstable(abi_unadjusted)]macro_rules! make{()=>{pub extern "unadjusted" fn active()->u8{7}}}make!();
