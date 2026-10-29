#![feature(no_core)]
#![no_core]
#![feature(compat_mod,compat_inner)]

extern crate base;
pub fn probe()->u8{base::gated::inner::marked()}
