#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(doc_cfg)]
#[doc(cfg(feature="not_enabled"))]pub fn answer()->u8{7}
