#![feature(no_core)]
#![no_core]
extern crate base;

fn wrong_scalar<const B: bool>() -> base::ExportedArray<u32, B> {
    loop {}
}
