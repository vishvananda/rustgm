#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_const)]
#[diagnostic::on_const(note="target")]pub fn wrong(){}
