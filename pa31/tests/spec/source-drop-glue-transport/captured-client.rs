#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bounded(value: usize) {
    unsafe {
        base::drop_glue(&mut *(value as *mut base::Bounded<'_, '_, u8, 3>));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn conditional(value: usize) {
    unsafe {
        base::drop_glue(&mut *(value as *mut base::Conditional<'_>));
    }
}
