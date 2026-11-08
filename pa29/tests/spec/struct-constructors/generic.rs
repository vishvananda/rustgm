#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::second(base::Pair(base::Wrap(7), 42)) { 42 => match base::pair(7u8).1 { 42 => 42, _ => 8 }, _ => 7 } }
