#![feature(no_core, marker_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed_start() -> u8 { (8u8..16u8).start }
#[unsafe(no_mangle)]
pub extern "C" fn observed_end() -> u8 { (8u8..16u8).end }
#[unsafe(no_mangle)]
pub extern "C" fn observed_inclusive_start() -> u8 { base::inclusive_start(&(23u8..=47u8)) }
#[unsafe(no_mangle)]
pub extern "C" fn observed_inclusive_end() -> u8 { base::inclusive_end(&(23u8..=47u8)) }
