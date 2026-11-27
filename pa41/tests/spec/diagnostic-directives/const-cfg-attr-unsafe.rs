#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_const)]
trait T{}
#[cfg_attr(all(),unsafe(diagnostic::on_const(note="used")))]impl T for u8{}
