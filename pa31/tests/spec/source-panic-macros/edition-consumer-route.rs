#![feature(no_core)]
#![no_core]
extern crate panic_provider;
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u32 { panic_provider::route!(tag [nested] {tokens}) }
