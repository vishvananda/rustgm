#![feature(no_core)]
#![no_core]

extern crate base;

struct Alpha;
struct Beta;

pub fn bad() -> u8 {
    base::check_pattern_dispatch::<*const Alpha, *const Beta>()
}
