#![feature(no_core,rustc_attrs)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
#[rustc_builtin_macro]
macro_rules! concat { () => {} }
#[rustc_builtin_macro]
macro_rules! stringify { () => {} }
macro_rules! concat { ($($t:tt)*) => {"shadowed"} }
#[rustc_diagnostic_item=concat!(unresolved!())]pub mod value {}
