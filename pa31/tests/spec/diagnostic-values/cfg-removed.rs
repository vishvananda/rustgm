#![feature(no_core,rustc_attrs)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
#[cfg(any())]
#[rustc_diagnostic_item=undefined!()]pub mod removed {}
#[rustc_diagnostic_item="kept"]pub mod kept {}
