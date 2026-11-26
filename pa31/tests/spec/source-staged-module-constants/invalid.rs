#![feature(no_core)]
#![no_core]
extern crate base;

pub fn experimental_value() -> u8 {
    base::EXPERIMENTAL
}
