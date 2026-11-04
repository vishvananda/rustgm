#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_unmatched_args)]
#[diagnostic::on_unmatched_args(note="target")]pub fn wrong(){}
