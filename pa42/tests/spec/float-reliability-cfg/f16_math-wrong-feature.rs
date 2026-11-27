#![allow(internal_features,unused_macros,dead_code,unexpected_cfgs)]
#![feature(cfg_target_has_atomic)]
#[cfg(target_has_reliable_f16_math)]fn selected(){}
