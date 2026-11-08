#![feature(no_core,rustc_attrs,auto_traits)]
#![no_core]
#![allow(internal_features)]
extern crate base;
#[rustc_diagnostic_item = "Send"]
pub unsafe auto trait Send {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
