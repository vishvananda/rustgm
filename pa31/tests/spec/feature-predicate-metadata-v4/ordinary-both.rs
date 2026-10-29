#![feature(no_core,predicate_api,predicate_probe)]
#![no_core]
extern crate base;
pub fn answer()->u8{base::gated(7u8)}
