#![feature(no_core)]
#![no_core]

extern crate base;
use base::Value;

fn requires_reference<T: base::RefOnly>(_: &T) -> u8 {
    29
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_bound() -> u8 {
    requires_reference(&(&1u8))
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_method() -> u8 {
    (&1u8).value()
}

#[cfg(reject_scalar)]
pub fn scalar() -> u8 {
    requires_reference(&1u8)
}
