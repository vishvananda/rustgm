#![feature(no_core,experimental_identity)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{base::routes::unstable_alias(7u8)}
