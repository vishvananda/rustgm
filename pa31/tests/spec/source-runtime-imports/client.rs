#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
unsafe extern "C" fn call_memcpy(
    d: *mut base::Void,
    s: *const base::Void,
    n: usize,
) -> *mut base::Void {
    unsafe { base::runtime::memcpy(d, s, n) }
}
#[unsafe(no_mangle)]
unsafe extern "C" fn call_memmove(
    d: *mut base::Void,
    s: *const base::Void,
    n: usize,
) -> *mut base::Void {
    unsafe { base::runtime::memmove(d, s, n) }
}
#[unsafe(no_mangle)]
unsafe extern "C" fn call_memset(d: *mut base::Void, c: i32, n: usize) -> *mut base::Void {
    unsafe { base::runtime::memset(d, c, n) }
}
#[unsafe(no_mangle)]
unsafe extern "C" fn call_memcmp(a: *const base::Void, b: *const base::Void, n: usize) -> i32 {
    unsafe { base::runtime::memcmp(a, b, n) }
}
#[unsafe(no_mangle)]
unsafe extern "C" fn call_bcmp(a: *const base::Void, b: *const base::Void, n: usize) -> i32 {
    unsafe { base::runtime::bcmp(a, b, n) }
}
#[unsafe(no_mangle)]
unsafe extern "C" fn call_strlen(s: *const i8) -> usize {
    unsafe { base::runtime::text_length(s) }
}
