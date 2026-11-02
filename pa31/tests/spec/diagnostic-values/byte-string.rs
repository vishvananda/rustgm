#![feature(no_core,rustc_attrs)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
macro_rules! name { () => {b"bytes"} }
#[rustc_diagnostic_item=name!()]pub mod value {}
