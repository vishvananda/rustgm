#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let o = base::some::<u8>()(41); match o { base::Opt::Some(41) => 42, _ => 7 } }
