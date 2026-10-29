#![feature(no_core)]
#![no_core]
extern crate base;
#[expect(deprecated)]use base::gated::marked as chosen;
pub fn probe()->u8{chosen()}
