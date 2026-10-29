#![feature(no_core)]
#![no_core]
#![feature(compat_mod,compat_const)]

extern crate base;
pub fn probe()->u8{const {base::gated::const_marked()}}
