#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let base::Wrap(x) = base::Wrap(base::open(base::wrap(42))); match base::wrap(x) { base::Wrap(42) => 42, base::Wrap(_) => 7 } }
