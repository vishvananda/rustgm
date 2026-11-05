#![feature(no_core)]
#![no_core]
extern crate base;
pub fn invalid() { let packet = base::make(7i32); let first = base::take(packet); let again = base::take(packet); }
