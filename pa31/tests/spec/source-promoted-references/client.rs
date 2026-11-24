#![feature(no_core)]
#![no_core]
extern crate base;

const MESSAGE: fn(u8) -> u8 = base::current().message;
#[unsafe(no_mangle)]
pub extern "C" fn observe() -> i32 {
    let table = base::current();
    if (table.retry)(12) { 1 } else { MESSAGE(11) as i32 }
}
