#![feature(no_core, asm_native_fixture)]
#![no_core]

extern crate base;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_bittest32(words: usize, bit: i32, output: usize) {
    unsafe {
        *(output as *mut u8) = base::word32::_bittest(words as *const i32, bit);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_bittestandset32(words: usize, bit: i32, output: usize) {
    unsafe {
        *(output as *mut u8) = base::word32::_bittestandset(words as *mut i32, bit);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_bittestandreset32(words: usize, bit: i32, output: usize) {
    unsafe {
        *(output as *mut u8) = base::word32::_bittestandreset(words as *mut i32, bit);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_bittestandcomplement32(words: usize, bit: i32, output: usize) {
    unsafe {
        *(output as *mut u8) = base::word32::_bittestandcomplement(words as *mut i32, bit);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_bittest64(words: usize, bit: i64, output: usize) {
    unsafe {
        *(output as *mut u8) = base::word64::_bittest64(words as *const i64, bit);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_bittestandset64(words: usize, bit: i64, output: usize) {
    unsafe {
        *(output as *mut u8) = base::word64::_bittestandset64(words as *mut i64, bit);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_bittestandreset64(words: usize, bit: i64, output: usize) {
    unsafe {
        *(output as *mut u8) = base::word64::_bittestandreset64(words as *mut i64, bit);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn observe_bittestandcomplement64(words: usize, bit: i64, output: usize) {
    unsafe {
        *(output as *mut u8) = base::word64::_bittestandcomplement64(words as *mut i64, bit);
    }
}
