#![feature(no_core)]
#![no_core]
extern crate base;

const RESULT: base::Maybe<u8> = base::carry(base::Maybe::Present(42));

fn read(value: base::Maybe<u8>) -> u8 {
    match value {
        base::Maybe::Present(item) => item,
        base::Maybe::Absent => 73,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_value() -> u8 {
    read(base::carry(base::Maybe::Present(73)))
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_const() -> u8 {
    read(RESULT)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_empty() -> u8 {
    read(base::carry(base::Maybe::Absent))
}
