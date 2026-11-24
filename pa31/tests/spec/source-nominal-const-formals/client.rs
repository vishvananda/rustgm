#![feature(no_core)]
#![no_core]

extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed_enum() -> u8 {
    base::enum_mode::<{ base::Order::Second }>()
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_record() -> u8 {
    base::record_mode::<{ base::Options { number: 41 } }>()
}

#[cfg(wrong_nominal)]
pub fn wrong() -> u8 {
    base::enum_mode::<{ base::Options { number: 41 } }>()
}
