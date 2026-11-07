#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let x = 42u8; let y = 7u8; let p = base::Pair { a: &x, b: &y }; match *p.first() { 42 => match *p.second() { 7 => 42, _ => 8 }, _ => 7 } }
