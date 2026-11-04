#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_cfg)]
#[unsafe(link(name="candidate",cfg(all())))]unsafe extern "C"{}
