#![feature(no_core)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
macro_rules! name { () => {"missing"} }
#[rustc_diagnostic_item=name!()]pub mod value {}
