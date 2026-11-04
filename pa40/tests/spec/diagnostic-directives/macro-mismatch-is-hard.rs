#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_unmatched_args)]
#[diagnostic::on_unmatched_args(note="custom mismatch")]macro_rules! m{()=>{7u8}}pub fn value()->u8{m!(wrong)}
