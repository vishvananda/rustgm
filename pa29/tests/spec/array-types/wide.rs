#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let a: [u8; 2] = base::unwide(base::wide([42u8, 0])); let w: base::Wide<bool, 1> = base::wide([true]); let p = &raw const a as *const u8; match unsafe { *p } { 42 => match w.items { [true] => 42, [false] => 7 }, _ => 7 } }
