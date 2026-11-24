#![feature(no_core, pointer_metadata_fixture)]
#![no_core]
extern crate base;

const LENGTH: usize = (&[1u8, 2, 3] as &[u8]).len();

#[unsafe(no_mangle)]
pub extern "C" fn observed_const() -> usize {
    LENGTH
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_runtime() -> usize {
    (&[1u8, 2, 3, 4, 5] as &[u8]).len()
}
