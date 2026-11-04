#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_cfg,cfg_ub_checks)]
#[link(name="candidate",cfg(ub_checks))]unsafe extern "C"{pub fn external()->u8;}
