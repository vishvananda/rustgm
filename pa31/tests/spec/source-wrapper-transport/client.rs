#![feature(no_core, c_variadic)]
#![no_core]
#![allow(unused_variables, unused_mut)]

extern crate base;

fn main() {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn variadic(value: i32, mut args: ...) -> i32 {
    value
}

#[unsafe(no_mangle)]
pub extern "C" fn pinned(value: u64) -> u64 {
    base::Pinned { pointer: value }.pointer
}

pub struct Local;

#[cfg(accepted)]
impl base::Slot for base::Pinned<Local> {}

#[cfg(nonfundamental)]
impl base::Slot for base::Ordinary<Local> {}
