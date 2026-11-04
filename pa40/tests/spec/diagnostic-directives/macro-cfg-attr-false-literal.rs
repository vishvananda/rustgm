#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_unmatched_args)]
#[cfg_attr(any(),diagnostic::on_unmatched_args(note="\q"))]macro_rules! value{()=>{7u8}}
pub fn answer()->u8{value!()}
