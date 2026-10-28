#![feature(identity_api)]
#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{base::bridge::forwarded(7u8)}
