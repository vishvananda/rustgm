#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::or(base::Opt::Some(1u8, 2u8), 2u8) { 1 => 42, _ => 7 } }
