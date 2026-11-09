#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let e = base::variant()(41); match e { base::E::A(41) => 42, _ => 7 } }
