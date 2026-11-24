#![feature(no_core)]
#![no_core]
extern crate base;

fn reject(text: *const u8) -> usize {
    unsafe { base::runtime::text_length(text) }
}
