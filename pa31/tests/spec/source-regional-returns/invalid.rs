#![feature(no_core)]
#![no_core]
extern crate base;

pub fn escape() -> &'static u8 {
    let local = 42;
    match base::shorten(base::Outcome::Value(&local)) {
        base::Outcome::Value(value) => value,
        base::Outcome::Error(_) => &0,
    }
}
