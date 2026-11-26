#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn obs_select() -> u32 { (base::SELECT)(7) }
#[unsafe(no_mangle)]
pub extern "C" fn obs_borrow() -> u32 { let value = 13; *(base::BORROW)(&value) }
#[unsafe(no_mangle)]
pub extern "C" fn obs_value() -> u32 { (base::VALUE)() }
#[unsafe(no_mangle)]
pub extern "C" fn obs_block() -> u32 { (base::BLOCK)() }
#[unsafe(no_mangle)]
pub extern "C" fn obs_check() -> u32 { base::with_check(23) }
#[unsafe(no_mangle)]
pub extern "C" fn obs_forward() -> u32 { let value = 31; *base::borrowed(&value) }
