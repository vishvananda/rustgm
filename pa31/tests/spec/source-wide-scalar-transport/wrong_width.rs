#![feature(no_core, f16, f128)]
#![no_core]
extern crate base;

pub fn wrong_width() -> f16 { base::quad_low() }
