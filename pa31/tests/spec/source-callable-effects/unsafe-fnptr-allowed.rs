#![feature(no_core)]
#![no_core]
#![feature(compat_mod)]
extern crate base;
pub fn probe()->u8{let value:unsafe fn()->u8=base::gated::unsafe_marked;unsafe{value()}}
