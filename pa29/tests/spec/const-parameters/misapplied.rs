#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let _r: base::Tag<6> = base::rigid::<7>(); 42 }
