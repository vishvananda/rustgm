#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match base::or(base::wrap(40u8), 1u8) { 40 => match base::first(base::two(42u8, 3u8)) { 42 => 42, _ => 8 }, _ => 7 }
}
