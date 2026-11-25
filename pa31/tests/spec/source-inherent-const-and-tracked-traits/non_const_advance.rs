#![feature(no_core, const_trait_impl, const_destruct)]
#![no_core]

extern crate base;

use base::{Acc, Step};

pub struct RuntimeOnly(pub u32);

impl Step for RuntimeOnly {
    fn next_val(self) -> Self {
        self
    }
}

pub const BAD: u32 = Acc { value: RuntimeOnly(1) }.advance().value.0;
