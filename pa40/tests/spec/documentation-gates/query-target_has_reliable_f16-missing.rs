#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(doc_cfg)]
#[doc(cfg(target_has_reliable_f16))]pub fn answer()->u8{7}
