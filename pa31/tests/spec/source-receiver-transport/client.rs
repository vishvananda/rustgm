#![feature(arbitrary_self_types)]
#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

struct Test;

impl base::Receiver for Test {
    type Target = u8;
}

fn observe<T: base::Receiver>(value: T::Target) -> T::Target
where
    T::Target: base::Sized,
{
    value
}

#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u8 {
    observe::<Test>(41)
}
