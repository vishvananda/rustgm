#![feature(no_core)]
#![no_core]
#![warn(unused_must_use)]
extern crate base;
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8 {base::identity(7u8);7u8}
