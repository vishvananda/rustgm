#![feature(no_core)]
#![allow(internal_features)]
#![no_core]
extern crate base;

fn identity(value: u32) -> u32 {
    value
}

pub fn call_hidden() -> u32 {
    let f: fn(u32) -> u32 = identity;
    f.hidden_fn_method()
}
