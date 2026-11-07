#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let o: base::Opt<u8> = base::Opt::Some(1u16);
    match base::or(o, 2u8) { 1 => 42, _ => 7 }
}
