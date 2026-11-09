#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let v = 42u8; let h = base::Held { text: &v }; unsafe { *h.text() as i32 } }
