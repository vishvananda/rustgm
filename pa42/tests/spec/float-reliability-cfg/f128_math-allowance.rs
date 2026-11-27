#![allow(internal_features,unused_macros,dead_code,unexpected_cfgs)]
#![feature(allow_internal_unstable)]
#[allow_internal_unstable(cfg_target_has_reliable_f16_f128)]macro_rules! route{()=>{cfg!(target_has_reliable_f128_math)}}
pub fn answer()->bool{route!()}
