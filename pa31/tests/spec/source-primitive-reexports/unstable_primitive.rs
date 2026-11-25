#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

pub fn private_reexport(x: base::primitive::u8) -> base::primitive::u8 {
    x
}
