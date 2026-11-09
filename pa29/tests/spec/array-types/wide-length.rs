#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let _w: base::Wide<u8, 3> = base::wide([42u8, 0]); 42 }
