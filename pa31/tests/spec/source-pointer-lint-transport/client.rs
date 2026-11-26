#![feature(no_core)]
#![no_core]

extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed_free(value: u8) -> u8 {
    let address = &value as *const u8;
    unsafe { *base::pointer(address) }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_method(value: u8) -> u8 {
    let wrapped = base::Pointer { value };
    unsafe { *wrapped.as_ptr() }
}
