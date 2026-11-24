#![no_std]
#![allow(internal_features)]
#![feature(no_core, lang_items)]
#![no_core]

extern crate base;

static TEXT: [i8; 8] = [114, 117, 115, 116, 103, 109, 33, 0];

#[unsafe(no_mangle)]
pub extern "C" fn observe_len() -> usize {
    base::measure_local_strlen(&raw const TEXT as *const i8)
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_diff() -> f64 {
    base::measure_module_safe_c(17.5, 5.25)
}
