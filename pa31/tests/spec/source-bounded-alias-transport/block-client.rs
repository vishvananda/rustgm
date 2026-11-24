#![feature(no_core)]
#![no_core]
extern crate base;

pub fn observed() -> u32 {
    base::block_word()
}
