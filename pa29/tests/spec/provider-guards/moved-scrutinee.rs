#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let o = base::Opt::Some(42u8);
    let _a = base::guard(o);
    match base::guard(o) { 42 => 42, _ => 7 }
}
