#![feature(no_core)]
#![no_core]
extern crate base;

pub struct NotCallable(pub u32);

pub fn bad(x: NotCallable) -> u32 {
    base::apply_once(x, 1)
}
