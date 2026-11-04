#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_cfg)]
#[link(name="candidate",cfg(target_has_atomic_load_store="8"))]unsafe extern "C"{pub fn external()->u8;}
