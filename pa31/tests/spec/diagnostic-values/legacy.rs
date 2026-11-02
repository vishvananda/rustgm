#![feature(no_core,rustc_attrs)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
#[rustc_builtin_macro]
macro_rules! concat { () => {} }
#[rustc_builtin_macro]
macro_rules! stringify { () => {} }
macro_rules! int_module { ($T:ident) => { #[rustc_diagnostic_item = concat!(stringify!($T), "_legacy_mod")]pub mod $T {} }; }
int_module!(u8);int_module!(i16);
