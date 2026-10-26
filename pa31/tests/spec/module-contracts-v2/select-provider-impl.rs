#![feature(no_core)]#![no_core]
extern crate base; fn prove<T:base::num::Sealed>() {} pub fn selected() { prove::<u8>(); }
