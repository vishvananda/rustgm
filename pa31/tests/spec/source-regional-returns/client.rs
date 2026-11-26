#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn good(value: u8) -> u8 {
    match base::forward(base::Outcome::Value(&value)) {
        base::Outcome::Value(result) => *result,
        base::Outcome::Error(code) => code,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn failed(code: u8) -> u8 {
    match base::shorten(base::Outcome::Error(code)) {
        base::Outcome::Value(result) => *result,
        base::Outcome::Error(code) => code,
    }
}
