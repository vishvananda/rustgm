#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_const)]
#![deny(malformed_diagnostic_attributes,misplaced_diagnostic_attributes)]
trait T{}
#[diagnostic::on_const()]impl T for u8{}
