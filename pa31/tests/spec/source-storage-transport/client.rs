#![feature(no_core)]
#![no_core]

extern crate base;

fn main() {}

fn frozen<T: base::Freeze>() {}

#[cfg(accepted)]
const ALLOWED: base::Hold<&'static u8> =
    base::Hold(base::Relax(&7u8));

#[cfg(accepted)]
#[unsafe(no_mangle)]
pub extern "C" fn accept(value: u8) -> u8 {
    frozen::<base::Ordinary<u8>>();
    let _value = ALLOWED;
    value
}

#[cfg(cell)]
pub fn reject_cell() {
    frozen::<base::Slot<u8>>();
}

#[cfg(pinned)]
pub fn reject_pinned() {
    frozen::<base::Pinned<u8>>();
}

#[cfg(dangling)]
const REJECTED: base::Ordinary<&'static u8> =
    base::Ordinary(unsafe { &*(1usize as *const u8) });

#[cfg(dangling)]
#[unsafe(no_mangle)]
pub extern "C" fn reject_dangling() {
    let _value = REJECTED;
}
