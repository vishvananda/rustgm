#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed(mut value: u8, mask: u8) -> u8 {
    let mut target = base::Pin { pointer: &mut value };
    let receiver = base::Pin { pointer: &mut target };
    base::Resume::resume(receiver, mask)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_shared(value: u8) -> u8 {
    let target = base::Read(&value);
    base::Read::read(base::Pin { pointer: &target })
}
