#![feature(no_core)]
#![no_core]
extern crate base;

pub struct NotIter(pub u32);

pub fn bad(x: NotIter) -> u32 {
    base::sum_iter(x)
}
