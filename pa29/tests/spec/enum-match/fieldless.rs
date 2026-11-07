#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match base::pick(base::second()) { 2 => match base::pick(base::first()) { 1 => 42, _ => 8 }, _ => 7 }
}
