#![feature(no_core)]
#![no_core]
extern crate base;

pub fn rejected() -> u32 {
    let s = base::Sealed { secret: 1 };
    unsafe { s.secret }
}
