#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let _c: base::Contra<u8> = base::contra(); let _i: base::Inv<[u8]> = base::inv_new(); 42 }
