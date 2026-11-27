#![feature(no_core, location_fixture)]
#![no_core]
extern crate base as core;
use core::clone::Clone;
use core::derive;
use core::marker::Copy;

#[derive(Copy, Clone)]
struct Packet {
    value: u8,
}

#[unsafe(no_mangle)]
pub extern "C" fn answer() -> u8 {
    let value = Packet { value: 53 };
    let cloned = Clone::clone(&value);
    cloned.value
}
