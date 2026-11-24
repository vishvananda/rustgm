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
    *(&41u8).value()
}

#[cfg(reject_scalar)]
pub fn scalar() -> u8 {
    requires_reference(&1u8)
}

struct Local;

impl<'a> base::Borrowed<'a, u8> for Local {
    fn inspect(&self, _: &'a u8) -> u8 {
        71
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_trait_lifetime() -> u8 {
    let local = Local;
    let value = 1u8;
    base::Borrowed::inspect(&local, &value)
}
