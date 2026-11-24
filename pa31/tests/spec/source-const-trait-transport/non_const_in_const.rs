#![no_std]
#![allow(internal_features)]
#![feature(no_core, lang_items, const_trait_impl)]
#![no_core]

extern crate base;

use base::Scale;

const BAD: u32 = base::Fixed(5).runtime_bonus();

pub fn observe() -> u32 {
    BAD
}
