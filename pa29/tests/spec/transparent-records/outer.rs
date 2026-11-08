#![feature(no_core)]
#![no_core]
extern crate base;
#[repr(transparent)]
struct Outer(base::Wrap, ());
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::unwrap(Outer(base::Wrap(42), ()).0) { 42 => 42, _ => 7 } }
