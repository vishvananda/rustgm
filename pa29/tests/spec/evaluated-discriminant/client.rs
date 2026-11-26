#![feature(no_core)]
#![no_core]

extern crate base;

const TAGS: (u8, u8) = (base::code(base::Header::Explicit), base::code(base::Header::Other));

#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match TAGS {
        (16, 17) => 42,
        _ => 1,
    }
}
