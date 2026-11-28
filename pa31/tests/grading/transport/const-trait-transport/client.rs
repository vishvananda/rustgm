#![no_std]
#![allow(internal_features)]
#![feature(no_core, lang_items, const_trait_impl)]
#![no_core]

extern crate base;

use base::{Scale, UnsafeScale};

const C_FIXED: u32 = base::Fixed(5).factor();
const C_DOUBLED: u32 = base::Fixed(5).doubled();
const C_TAG: u32 = base::apply_tag(base::Fixed(5));
const C_UNSAFE: u32 = base::Fixed(5).raw_factor();
const C_REF: u32 = {
    let x = 7;
    base::RefScale(&x).factor()
};
const C_STRICT: u32 = base::strict_scale(base::Fixed(4));
const C_TWICE: u32 = base::scale_twice(base::Fixed(6));

#[unsafe(no_mangle)]
pub extern "C" fn observe_const_fixed() -> u32 {
    C_FIXED
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_const_doubled() -> u32 {
    C_DOUBLED
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_const_tag() -> u32 {
    C_TAG
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_const_unsafe() -> u32 {
    C_UNSAFE
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_const_ref() -> u32 {
    C_REF
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_const_strict() -> u32 {
    C_STRICT
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_const_twice() -> u32 {
    C_TWICE
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_dyn_factor() -> u32 {
    base::Dynamic(9).factor()
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_dyn_doubled() -> u32 {
    base::Dynamic(9).doubled()
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_fixed_bonus() -> u32 {
    base::Fixed(5).runtime_bonus()
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_dyn_bonus() -> u32 {
    base::Dynamic(9).runtime_bonus()
}
