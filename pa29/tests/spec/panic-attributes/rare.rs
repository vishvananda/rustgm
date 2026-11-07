#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::rare(42) { 42 => match base::rarer(42) { 42 => match base::relay(42) { 42 => 42, _ => 9 }, _ => 8 }, _ => 7 } }
