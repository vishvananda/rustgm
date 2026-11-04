#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(doc_cfg)]
macro_rules! make{($m:meta)=>{#[doc($m)]pub fn answer()->u8{7}}}make!(cfg(any()));
