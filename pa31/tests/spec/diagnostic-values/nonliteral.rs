#![feature(no_core,rustc_attrs)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
const NAME:&str="not_literal";
#[rustc_diagnostic_item=NAME]pub mod value {}
