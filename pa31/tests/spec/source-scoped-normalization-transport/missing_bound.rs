#![feature(no_core)]
#![no_core]
extern crate base;

pub fn bad() -> u32 {
    base::const_hrtb_passthrough::<base::Plain>(1)
}
