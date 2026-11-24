#![feature(no_core)]
#![no_core]

extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed_equal(value: u8) -> bool {
    base::Key(value) == base::Key(7u8)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_unequal(value: u8) -> bool {
    base::Key(value) != base::Key(7u8)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_different_rhs(value: u8) -> bool {
    base::Key(0u8) != base::Right(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_less(value: u8) -> bool {
    base::Key(value) < base::Key(7u8)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_greater_equal(value: u8) -> bool {
    base::Key(value) >= base::Key(7u8)
}
