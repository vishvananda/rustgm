#![feature(no_core)]
#![no_core]

extern crate base;
pub fn probe()->u8{base::gated::unsafe_marked()}
