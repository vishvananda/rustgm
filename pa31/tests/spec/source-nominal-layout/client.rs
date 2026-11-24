#![feature(no_core)]
#![no_core]

extern crate base;

fn main() {}

#[cfg(accepted)]
#[unsafe(no_mangle)]
pub extern "C" fn read_aligned(value: u8) -> u8 {
    base::aligned(value).0
}

#[cfg(accepted)]
#[unsafe(no_mangle)]
pub extern "C" fn read_packed() -> u32 {
    base::packed().rest
}

#[cfg(accepted)]
#[unsafe(no_mangle)]
pub extern "C" fn read_indirect(argument: base::Indirect) -> u64 {
    argument.0
}

#[cfg(unaligned)]
#[unsafe(no_mangle)]
pub extern "C" fn packed_reference(value: &base::Packed) -> &u32 {
    &value.rest
}
