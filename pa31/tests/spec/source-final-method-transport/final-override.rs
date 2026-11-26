#![feature(no_core)]
#![no_core]

extern crate base;

struct Local;

impl base::Score for Local {
    fn fixed(&self) -> u8 {
        7
    }
}
