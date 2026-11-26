#![feature(no_core)]
#![no_core]
#![deny(deprecated)]
extern crate base;
pub fn read() -> u8 { base::legacy_alias() }
