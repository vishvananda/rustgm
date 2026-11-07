#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let mut hit = 7u8;
    base::apply_mut(move |x| { hit = x; hit }, 42u8) as i32
}
