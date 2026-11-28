#![feature(no_core, offset_transport)]
#![no_core]
extern crate base;

const MIDDLE: usize = base::field_offset::<base::Fields>(1);
const LAST: usize = base::field_offset::<base::Fields>(2);

#[unsafe(no_mangle)]
pub extern "C" fn observed_const() -> usize {
    MIDDLE
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_runtime() -> usize {
    LAST
}
