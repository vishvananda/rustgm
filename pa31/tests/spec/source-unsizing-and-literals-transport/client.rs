#![feature(no_core)]
#![no_core]

extern crate base;

use base::{
    SubVal, Val, byte_literal_sum, cstr_first_and_len, match_raw_bytes, temp_guard_and_mut,
    unsize_bound, unsize_nominal, unsize_object, unsize_slice_sum, unsize_upcast,
};

struct Item(u32);
impl Val for Item {
    fn val(&self) -> u32 {
        self.0 + 10
    }
}
impl SubVal for Item {
    fn extra(&self) -> u32 {
        self.0
    }
}

const CONST_BYTES: u32 = byte_literal_sum();

#[unsafe(no_mangle)]
pub extern "C" fn obs_temp() -> u32 {
    temp_guard_and_mut(10)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_bytes() -> u32 {
    CONST_BYTES + match_raw_bytes(b"\xff\x02") + match_raw_bytes(b"\x00\x01")
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_cstr() -> u32 {
    cstr_first_and_len()
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_slice() -> u32 {
    unsize_slice_sum(3, 4, 5)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_object() -> u32 {
    let item = Item(7);
    unsize_object(&item)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_upcast() -> u32 {
    let item = Item(5);
    let sub: &dyn SubVal = &item;
    unsize_upcast(sub)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_bound() -> u32 {
    let item = Item(9);
    let erased: &dyn Val = unsize_bound(&item);
    erased.val()
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_nominal() -> u32 {
    let item = Item(12);
    unsize_nominal(&item)
}
