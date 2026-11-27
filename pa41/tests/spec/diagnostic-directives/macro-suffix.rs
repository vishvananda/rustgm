#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_unmatched_args)]
#[diagnostic::on_unmatched_args(note="x"u8)]macro_rules! value{()=>{7u8}}
pub fn answer()->u8{value!()}
