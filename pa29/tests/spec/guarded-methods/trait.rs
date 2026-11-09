#![feature(no_core)]
#![no_core]
extern crate base;
use base::Get;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { unsafe { base::Key(42).fetch() as i32 } }
