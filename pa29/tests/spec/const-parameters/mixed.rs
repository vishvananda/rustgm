#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match (base::Mixed::<7, u8> { value: 40 }).get() { 40 => match base::wrap::<9>(42).get() { 42 => 42, _ => 8 }, _ => 7 } }
