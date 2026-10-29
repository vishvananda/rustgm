#![feature(no_core)]
#![no_core]
#![allow(deprecated)]
extern crate base;
pub fn probe()->u8{base::gated::marked()}
