#![feature(no_core)]
#![no_core]

extern crate base;

struct Local;

impl base::Score for Local {
    fn open(&self) -> u8 {
        73
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_final() -> u8 {
    <u8 as base::Score>::fixed(&0)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_override() -> u8 {
    <Local as base::Score>::open(&Local)
}
