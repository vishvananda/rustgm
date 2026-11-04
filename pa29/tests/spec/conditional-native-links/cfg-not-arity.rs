#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_cfg)]
#[link(name="candidate",cfg(not(unix,windows)))]unsafe extern "C"{pub fn external()->u8;}
