#![feature(no_core)]
#![no_core]
extern crate base;
static N: u8 = 42;
static P: base::Pair<u8> = base::Pair { a: 42, b: 0 };
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match (N, P.a) { (42, 42) => 42, _ => 7 } }
