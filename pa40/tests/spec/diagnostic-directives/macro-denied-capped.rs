#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_unmatched_args)]
#![deny(malformed_diagnostic_attributes,misplaced_diagnostic_attributes)]
#[diagnostic::on_unmatched_args()]macro_rules! value{()=>{7u8}}
pub fn answer()->u8{value!()}
