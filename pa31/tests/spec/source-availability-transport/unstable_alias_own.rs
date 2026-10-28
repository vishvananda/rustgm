#![feature(no_core,alias_unstable)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{base::routes::unstable_alias(7u8)}
