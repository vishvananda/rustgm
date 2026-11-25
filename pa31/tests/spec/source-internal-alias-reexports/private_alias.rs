#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

pub fn leak_internal(pair: base::Pair) -> u32 {
    pair.0
}
