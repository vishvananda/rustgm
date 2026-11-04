#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_const)]
trait T{}
#[diagnostic::on_const(message=7)]impl T for u8{}
