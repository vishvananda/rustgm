#![feature(no_core)]
#![no_core]

extern crate base;

struct NonConst;

impl base::ConstSeed for NonConst {
    fn seed() -> u32 {
        9
    }
}

pub const BAD: u32 = base::Wrap::<NonConst>::ONE;
