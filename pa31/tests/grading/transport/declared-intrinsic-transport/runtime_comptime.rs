#![no_std]
#![allow(internal_features)]
#![feature(no_core, lang_items)]
#![no_core]

extern crate base;

pub fn call_comptime_at_runtime() -> u32 {
    base::comptime_free()
}
