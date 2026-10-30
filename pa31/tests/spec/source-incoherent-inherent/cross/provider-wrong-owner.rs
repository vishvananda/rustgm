#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#[rustc_has_incoherent_inherent_impls]
pub struct Granted;
pub struct Token(pub u8);
