#![feature(no_core)]
#![no_core]
extern crate base;

pub fn observed() -> base::Exported<u8, u8> {
    34
}

pub fn provider() -> u32 {
    base::provider_value()
}
