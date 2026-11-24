#![feature(fn_ptr_trait)]
#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

fn generic<F: base::FnPtr>(pointer: F) -> *const () {
    pointer.addr()
}

unsafe extern "C" {
    fn provide_pointer() -> extern "C" fn() -> u8;
    fn observe_pointer(address: *const ()) -> u8;
}

#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u8 {
    let pointer = unsafe { provide_pointer() };
    let address = generic(pointer);
    unsafe { observe_pointer(address) }
}
