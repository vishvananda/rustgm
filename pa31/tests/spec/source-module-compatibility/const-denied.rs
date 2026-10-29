#![feature(no_core)]
#![no_core]
#![feature(compat_mod)]

extern crate base;
pub fn probe()->u8{const {base::gated::const_marked()}}
