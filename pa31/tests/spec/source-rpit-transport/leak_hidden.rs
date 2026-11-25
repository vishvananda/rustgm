#![feature(no_core)]
#![no_core]

extern crate base;

pub fn leak() -> base::Visible {
    base::make_visible(7)
}
