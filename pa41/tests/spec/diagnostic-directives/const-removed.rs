#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_const)]
#[cfg(any())]trait T{}
#[unsafe(diagnostic::on_const(note="removed"))]impl T for u8{}
