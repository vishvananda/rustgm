#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(strict_provenance_lints)]
#![forbid(implicit_provenance_casts)]
pub fn round_trip(p:*const u8)->*const u8{let a=p as usize;a as *const u8}
