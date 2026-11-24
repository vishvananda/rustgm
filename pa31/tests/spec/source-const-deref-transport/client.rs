#![feature(no_core, full_drop_fixture, const_trait_impl, const_convert)]
#![no_core]
extern crate base;
use base::ops::{Deref, DerefMut};

const fn const_deref() -> u8 {
    let mut n = 29u8;
    let mut pointer = &mut n;
    *<&mut u8 as DerefMut>::deref_mut(&mut pointer) = 73;
    *<&mut u8 as Deref>::deref(&pointer)
}

const RESULT: u8 = const_deref();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_ordinary(value: usize) {
    unsafe { *(value as *mut u8) = RESULT; }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_glue(value: usize) {
    let pointer = unsafe { &mut *(value as *mut u8) };
    base::pin_store(&mut *pointer);
    let pinned = unsafe { base::pin::Pin::new_unchecked(pointer) };
    let mut pointer = unsafe { base::pin::Pin::take_inner(pinned) };
    let slot = <&mut u8 as DerefMut>::deref_mut(&mut pointer);
    *slot = match *slot { 41 => 42, _ => 99 };
}

#[cfg(ordinary_deref)]
struct Ordinary(u8);

#[cfg(ordinary_deref)]
impl Deref for Ordinary {
    type Target = u8;

    fn deref(&self) -> &u8 { &self.0 }
}

#[cfg(ordinary_deref)]
const BAD: u8 = *Ordinary(57).deref();
