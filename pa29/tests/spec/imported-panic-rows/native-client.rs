#![feature(no_core)]
#![no_core]
extern crate renamed;
use renamed::unreachable;
// The consumer's own `panic` module never captures `$crate::panic`.
pub mod panic {}
#[unsafe(no_mangle)]
pub extern "C" fn explicit() -> u32 {
    renamed::panic!()
}
#[unsafe(no_mangle)]
pub extern "C" fn unreached() -> u32 {
    unreachable!()
}
#[unsafe(no_mangle)]
pub extern "C" fn unreached_formatted() -> u32 {
    unreachable!("at {}", 3)
}
#[unsafe(no_mangle)]
pub extern "C" fn skipped() -> u32 {
    renamed::fail_with!()
}
