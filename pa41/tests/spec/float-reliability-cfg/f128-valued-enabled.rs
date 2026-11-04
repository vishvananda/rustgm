#![allow(internal_features,unused_macros,dead_code,unexpected_cfgs)]
#![feature(cfg_target_has_reliable_f16_f128)]
#[cfg(target_has_reliable_f128="unavailable")]fn selected(){}
