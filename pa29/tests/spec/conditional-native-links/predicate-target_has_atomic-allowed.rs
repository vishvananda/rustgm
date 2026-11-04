#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_cfg,cfg_target_has_atomic)]
#[link(name="candidate",cfg(target_has_atomic_load_store="8"))]unsafe extern "C"{pub fn external()->u8;}
