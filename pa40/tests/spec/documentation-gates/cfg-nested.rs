#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(doc_cfg)]
#[doc(cfg(all(unix,any(target_os="linux",target_os="macos"))))]pub fn answer()->u8{7}
