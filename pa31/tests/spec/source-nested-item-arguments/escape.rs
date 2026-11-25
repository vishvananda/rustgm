#![feature(no_core)]
#![no_core]
extern crate base;

pub fn escape<'a>(x: &'a u32) -> &'static u32 { base::choose_refs(x, x, true) }
