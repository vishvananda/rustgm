#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_ordinary(value: usize) {
    unsafe { base::ordinary(&mut *(value as *mut base::ProviderOwned<u8, u16>)); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_glue(value: usize) {
    unsafe { base::drop_glue(&mut *(value as *mut base::ProviderOwned<u8, u16>)); }
}
