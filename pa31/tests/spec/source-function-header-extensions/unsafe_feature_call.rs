#![feature(no_core)]
#![no_core]

extern crate base;

pub fn missing_target_feature() -> u32 {
    base::safe_add(1, 2)
}
