#![no_std]
#![no_main]
#![crate_type = "rlib"]
#[unsafe(no_mangle)]
pub extern "C" fn guest_answer() -> u8 { 9 }
