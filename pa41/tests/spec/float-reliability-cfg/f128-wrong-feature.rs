#![allow(internal_features,unused_macros,dead_code,unexpected_cfgs)]
#![feature(cfg_target_has_atomic)]
#[cfg(target_has_reliable_f128)]fn selected(){}
