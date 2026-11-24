#![feature(no_core)]
#![no_core]
extern crate base;

const REJECTED: usize = unsafe { base::runtime::text_length(0 as *const i8) };

pub fn use_value() -> usize {
    REJECTED
}
