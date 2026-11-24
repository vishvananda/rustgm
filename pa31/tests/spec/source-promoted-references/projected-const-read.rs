#![feature(no_core)]
#![no_core]
extern crate base;

const REJECTED: *mut base::Ops = base::STORED.slot.item.0;
#[unsafe(no_mangle)]
pub extern "C" fn observe() -> i32 {
    let table = unsafe { &*REJECTED };
    if (table.retry)(12) { 1 } else { 34 }
}
