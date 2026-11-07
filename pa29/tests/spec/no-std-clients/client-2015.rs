#![no_std]
// Rust 2015 names the injected `core` as a crate-root item, so a `use` path
// and `crate::core` both reach it.
use core::base;
fn pick(value: Option<u32>) -> u32 {
    match value {
        Option::Some(_) => forty_two!(),
        Option::None => 0,
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn nostd_prelude() -> u32 {
    pick(Option::Some(base()))
}
#[unsafe(no_mangle)]
pub extern "C" fn nostd_path() -> u32 {
    crate::core::base()
}
