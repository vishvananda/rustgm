#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let r; { let x = 42u8; r = base::Held(&x).get(); } match *r { 42 => 42, _ => 7 } }
