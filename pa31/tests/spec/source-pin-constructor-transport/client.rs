#![feature(no_core, full_drop_fixture)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_ordinary(value: usize) {
    let _ = value;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_glue(value: usize) {
    let pinned = unsafe { base::pin::Pin::new_unchecked(&mut *(value as *mut u8)) };
    let pointer = unsafe { base::pin::Pin::take_inner(pinned) };
    *pointer = 42;
}

#[cfg(wrong_pointer)]
pub unsafe fn wrong_pointer() {
    let _ = unsafe { base::pin::Pin::new_unchecked(41u8) };
}
