#![feature(no_core)]
#![no_core]
extern crate base;

pub fn hidden_line() -> u32 {
    base::hidden::inner::line!()
}
