#![feature(no_core)]
#![no_core]
extern crate base;

pub fn rejected() -> u32 {
    base::private_tracked().line()
}
