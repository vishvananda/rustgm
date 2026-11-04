#![allow(internal_features,unused_macros,dead_code,unexpected_cfgs)]
macro_rules! route{($p:meta)=>{#[cfg($p)]pub fn selected(){}}}route!(target_has_reliable_f16);
