#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::wrap(1u8) { base::Opt::Some(_) => 42 } }
