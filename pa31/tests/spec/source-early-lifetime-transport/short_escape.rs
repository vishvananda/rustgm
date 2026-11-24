#![no_std]
#![allow(internal_features)]
#![feature(no_core, lang_items)]
#![no_core]

extern crate base;

pub fn bad<'a, 'b>(x: &'a u8, y: &'b u8) -> &'b u8 {
    base::shorten(x, y)
}
