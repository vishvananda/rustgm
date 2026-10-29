#![feature(no_core)]
#![no_core]
#![deny(deprecated)]
extern crate base;
pub fn probe()->u8{base::gated::marked()}
