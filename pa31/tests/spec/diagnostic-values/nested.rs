#![feature(no_core,rustc_attrs)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
#[rustc_builtin_macro]
macro_rules! concat { () => {} }
#[rustc_builtin_macro]
macro_rules! stringify { () => {} }
#[rustc_diagnostic_item=concat!(concat!("pre", "fix"), stringify!(u8))]pub mod value {}
