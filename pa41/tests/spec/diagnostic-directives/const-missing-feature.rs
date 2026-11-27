#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
trait T{}
#[diagnostic::on_const(note="ignored")]impl T for u8{}
