#![feature(no_core)]
#![no_core]
#![feature(dependent_api)]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed_explicit() -> u8 { base::dependent() }

#[unsafe(no_mangle)]
pub extern "C" fn observed_implied() -> u8 { base::chained() }
