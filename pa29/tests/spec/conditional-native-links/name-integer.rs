#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_cfg)]
#[link(name=42)]unsafe extern "C"{pub fn external()->u8;}
