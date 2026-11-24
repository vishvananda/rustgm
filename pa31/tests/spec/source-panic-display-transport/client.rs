#![feature(no_core)]
#![no_core]

extern crate base;

fn main() {}

#[unsafe(no_mangle)]
pub fn fresh_display(value: &u8) -> ! {
    base::panic_display(value)
}

#[cfg(missing_display)]
pub fn missing_display(value: &u16) -> ! {
    base::panic_display(value)
}

#[unsafe(no_mangle)]
pub const fn fresh_const_format(arguments: base::Arguments<'_>) -> ! {
    base::const_format_stop(arguments)
}

#[cfg(const_display)]
pub const REACHED: u8 = base::panic_display(&"imported display panic");

#[cfg(const_nonstring)]
pub const NONSTRING: u8 = base::panic_display(&13u8);
