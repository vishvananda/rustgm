#![feature(no_core)]
#![no_core]
extern crate base;

#[repr(C)]
struct D {
    mark: u8,
}

unsafe extern "C" {
    fn record_drop(mark: u8);
}

impl base::Drop for D {
    fn drop(&mut self) {
        unsafe { record_drop(self.mark); }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_ordinary(value: usize) {
    unsafe { base::ordinary(&mut *(value as *mut D)); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_glue(value: usize) {
    unsafe { base::drop_glue(&mut *(value as *mut D)); }
}
