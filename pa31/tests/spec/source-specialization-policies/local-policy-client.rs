#![feature(min_specialization, no_core)]
#![no_core]

extern crate base;

struct Local;

unsafe impl base::Applicable for Local {}

trait LocalCompared {
    fn answer() -> u8;
}

impl<T, const N: usize> LocalCompared for [T; N] {
    default fn answer() -> u8 {
        1
    }
}

impl<T: base::Applicable<T>, const N: usize> LocalCompared for [T; N] {
    fn answer() -> u8 {
        2
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn specialize_locally() -> u8 {
    <[Local; 2] as LocalCompared>::answer()
}
