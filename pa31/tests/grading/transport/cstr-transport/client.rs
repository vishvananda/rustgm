#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

unsafe extern "C" {
    fn check_bytes(pointer: *const i8) -> u8;
}

#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u8 {
    let literal: &base::CStr = c"41";
    unsafe { check_bytes(literal as *const base::CStr as *const i8) }
}
