#![no_std]
#![allow(internal_features)]
#![feature(no_core, lang_items)]
#![no_core]

extern crate base;

const C_FREE: u32 = base::comptime_free();
const C_METHOD: u32 = base::Holder(77).comptime_inner();

#[unsafe(no_mangle)]
pub extern "C" fn observe_comptime_free() -> u32 {
    C_FREE
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_comptime_method() -> u32 {
    C_METHOD
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_ctpop() -> u32 {
    base::pop_count(0b101101)
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_transmute() -> u32 {
    base::transmute_u32(19)
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_black_box() -> u32 {
    base::black_boxed(23)
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_unchecked_sub() -> u32 {
    base::unchecked_diff(30, 11)
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_sub_ok() -> u32 {
    let (value, overflowed) = base::overflow_diff(20, 7);
    if overflowed { 100 } else { value }
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_sub_overflow() -> u32 {
    let (value, overflowed) = base::overflow_diff(2, 5);
    if overflowed { 100 } else { value }
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_select() -> u32 {
    base::select_hint(true, 55, 66)
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_hinted() -> u32 {
    base::hinted(42u32)
}
