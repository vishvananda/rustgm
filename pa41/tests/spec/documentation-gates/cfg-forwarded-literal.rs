#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(doc_cfg)]
macro_rules! make{($s:literal)=>{#[doc(cfg(feature=$s))]pub fn answer()->u8{7}}}make!("not_enabled");
