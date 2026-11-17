#![feature(decl_macro)]
#![no_std]
#![no_main]
mod local { pub macro seven() { 7u8 } }
pub fn value() -> u8 { ::local::seven!() }
