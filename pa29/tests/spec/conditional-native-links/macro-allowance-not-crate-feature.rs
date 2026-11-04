#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(allow_internal_unstable)]
#[allow_internal_unstable(link_cfg)]macro_rules! make{()=>{#[link(name="candidate",cfg(all()))]unsafe extern "C"{pub fn external()->u8;}}}make!();
