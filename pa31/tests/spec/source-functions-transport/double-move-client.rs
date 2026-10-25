#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)] pub extern "C" fn answer() -> u8 { let x = 7u8; let first = base::identity(x); base::identity(x) }
