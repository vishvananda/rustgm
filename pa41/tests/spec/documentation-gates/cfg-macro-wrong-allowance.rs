#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(allow_internal_unstable)]
#[allow_internal_unstable(f16)]macro_rules! make{()=>{#[doc(cfg(any()))]pub fn answer()->u8{7}}}make!();
