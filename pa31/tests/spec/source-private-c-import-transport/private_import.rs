#![no_std]
#![allow(internal_features)]
#![feature(no_core, lang_items)]
#![no_core]

extern crate base;

pub fn leak() -> f64 {
    base::imp::helper_diff(1.0, 2.0)
}
