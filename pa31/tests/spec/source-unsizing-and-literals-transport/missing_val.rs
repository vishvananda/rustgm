#![feature(no_core)]
#![no_core]

extern crate base;

struct Plain(u32);

pub fn rejected(p: &Plain) -> u32 {
    base::unsize_nominal(p)
}
