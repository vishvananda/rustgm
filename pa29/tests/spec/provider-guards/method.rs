#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match base::Opt::Some(42u8).big() {
        true => match base::Opt::Some(1u8).big() { false => 42, true => 8 },
        false => 7,
    }
}
