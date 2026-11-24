#![feature(no_core)]
#![no_core]

extern crate base;

struct Local;

impl base::Choose for Local {
    fn amount(_: ()) -> u8 {
        41
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_scalar() -> u8 {
    let _: base::Bytes = base::Bytes { bytes: [0u8, 0u8] };
    19
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_record() -> u8 {
    <Local as base::Choose<{ base::Options { number: 41, enabled: true } }>>::amount(())
}

#[cfg(wrong_record)]
pub fn wrong() -> u8 {
    <Local as base::Choose<{ base::Options { number: 41, enabled: false } }>>::amount(())
}
