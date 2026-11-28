#![feature(no_core, promotion_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

fn borrowed() -> &'static u8 { &base::word() }
#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u8 { *borrowed() }
