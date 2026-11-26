#![feature(no_core)]
#![no_core]

extern crate base;

unsafe extern "C" {
    fn observe(value: u8);
}

fn main() {
    // Provider's concrete impl overrides `Overridable::select` to return `second` (7)
    // instead of the provided default body (which returns `first` = 1).
    let overridden = <u8 as base::Overridable>::select(1u8, 7u8);
    let carried = <u8 as base::Holder<u8>>::carry::<u8>(overridden);
    unsafe {
        observe(carried);
    }
}
