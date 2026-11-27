#![feature(no_core)]
#![no_core]
extern crate base;

pub fn use_without_feature() -> u8 {
    base::exposed()
}
