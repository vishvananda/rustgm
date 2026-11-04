#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_const)]
trait T{}
#[cfg_attr(any(),unsafe(diagnostic::on_const(note="unused")))]impl T for u8{}
