#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_const)]
trait T{}
#[unsafe(diagnostic::on_const(note="x"))]impl T for u8{}
