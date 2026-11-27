#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(doc_cfg,cfg_sanitize)]
#[doc(cfg(sanitize="address"))]pub fn answer()->u8{7}
