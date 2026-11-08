#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::Sync>() {}
fn lift<T: base::Sync>() { need::<base::Pair<T>>() }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { lift::<u8>(); 42 }
