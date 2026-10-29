#![feature(no_core,dispatch_api,dispatch_probe)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{base::dispatch(7u8)}
