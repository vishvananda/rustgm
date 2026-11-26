#![feature(no_core, experimental_constant)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn stable_value() -> u8 {
    base::STABLE
}

#[unsafe(no_mangle)]
pub extern "C" fn experimental_value() -> u8 {
    base::EXPERIMENTAL
}

#[unsafe(no_mangle)]
pub extern "C" fn nested_value() -> u8 {
    base::nested::VALUE
}
