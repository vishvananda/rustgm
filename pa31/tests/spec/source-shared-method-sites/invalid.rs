#![feature(no_core)]
#![no_core]
extern crate base;
pub fn wrong_argument()->u32 { base::through(&base::Outer(base::Value(11)),&true,true) }
