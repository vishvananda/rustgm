#![feature(decl_macro)]
#![no_std]
#![no_main]
#[macro_export] macro_rules! local { () => { 7u8 }; }
pub fn value() -> u8 { ::local!() }
