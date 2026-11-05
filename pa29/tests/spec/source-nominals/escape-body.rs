#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn invalid() -> i32 {
    let escaped: base::Packet<&'static i32>;
    { let local = 7i32; escaped = base::make(&local); }
    *base::take(escaped)
}
