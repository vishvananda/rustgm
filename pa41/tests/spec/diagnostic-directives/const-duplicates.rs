#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_const)]
trait T{}
#[diagnostic::on_const(message="first",message="last",note="one",note="two")]impl T for u8{}
