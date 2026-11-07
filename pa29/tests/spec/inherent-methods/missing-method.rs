#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let o = base::Opt::Some(5u8);
    match o.is_none() { true => 7, false => 42 }
}
