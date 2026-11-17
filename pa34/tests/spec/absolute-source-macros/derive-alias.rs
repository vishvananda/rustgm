#![feature(decl_macro)]
#![no_std]
#![no_main]
#[::first::derive_alias()]
pub enum Dir {}
pub fn value() -> u8 { 7 }
