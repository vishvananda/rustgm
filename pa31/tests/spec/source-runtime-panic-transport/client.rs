#![feature(no_core)]
#![no_core]
extern crate base;

fn main() {}

#[unsafe(no_mangle)]
pub fn fresh_fmt(arguments: base::Arguments<'_>) -> ! {
    base::panic_fmt(arguments)
}

#[unsafe(no_mangle)]
pub fn fresh_nounwind() -> ! {
    base::panic_nounwind("source panic client")
}

#[cfg(wrong_argument)]
pub fn wrong_argument() -> ! {
    base::panic_fmt("unexpected slice")
}
