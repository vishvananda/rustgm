#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(doc_cfg,cfg_target_has_atomic)]
#[doc(cfg(target_has_atomic_load_store="8"))]pub fn answer()->u8{7}
