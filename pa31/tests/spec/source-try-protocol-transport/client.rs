#![feature(no_core)]
#![no_core]
extern crate base;

fn forward(value: base::Outcome<u8>) -> base::Outcome<u8> {
    let output = value?;
    base::Outcome::Value(output)
}

fn read(value: base::Outcome<u8>) -> u8 {
    match forward(value) {
        base::Outcome::Value(out) => out,
        base::Outcome::Error(error) => error,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_continue() -> u8 {
    read(base::Outcome::Value(41u8))
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_break() -> u8 {
    read(base::Outcome::Error(23u8))
}
