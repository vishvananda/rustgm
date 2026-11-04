#![allow(internal_features,unused_macros,dead_code,unexpected_cfgs)]
#![feature(cfg_target_has_reliable_f16_f128)]
macro_rules! route{($p:meta)=>{#[cfg($p)]pub fn selected(){}}}route!(target_has_reliable_f16);
