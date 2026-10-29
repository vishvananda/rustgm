#![feature(no_core)]
#![no_core]
extern crate base;
#[expect(deprecated)]
pub fn probe()->u8{base::gated::marked()}
