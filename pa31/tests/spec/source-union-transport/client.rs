#![feature(no_core)]
#![no_core]

extern crate base;

fn main() {}

#[cfg(accepted)]
#[unsafe(no_mangle)]
pub extern "C" fn read(x: u64) -> u64 {
    let word = base::Packet { whole: x };
    unsafe { word.whole }
}

#[cfg(accepted)]
#[unsafe(no_mangle)]
pub extern "C" fn delayed(x: u64) -> u64 {
    let word = base::Deferred { value: x };
    unsafe { word.value }
}

#[cfg(accepted)]
#[unsafe(no_mangle)]
pub extern "C" fn wrapped(x: u64) -> u64 {
    let word = base::Wrapped { value: base::Hold(x) };
    unsafe { word.value.0 }
}

#[cfg(accepted)]
#[unsafe(no_mangle)]
pub extern "C" fn closed(x: u64) -> u64 {
    let word = base::Closed { value: base::Hold(x) };
    unsafe { word.value.0 }
}

#[cfg(accepted)]
#[unsafe(no_mangle)]
pub extern "C" fn copied(x: u64) -> u64 {
    let word = base::Copied { value: x };
    unsafe { word.value }
}

#[cfg(unguarded)]
pub fn unguarded(x: base::Packet) -> u64 {
    x.whole
}

#[cfg(two_fields)]
pub fn two_fields(whole: u64, first: u8) -> base::Packet {
    base::Packet { whole, first }
}
