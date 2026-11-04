#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_cfg)]
#[link(name="missing_native",cfg(any()))]unsafe extern "C"{pub fn external()->u8;}pub fn answer()->u8{external()}
