#![feature(no_core,rustc_attrs)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
macro_rules! install { ($v:expr) => { #[rustc_diagnostic_item=$v]pub mod value {} }; }
macro_rules! name { () => {"forwarded"} }
install!(name!());
