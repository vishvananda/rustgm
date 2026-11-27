#![feature(no_core)]
#![no_core]
extern crate base;

pub fn escape(value: u8) -> <&'static u16 as base::Rebind<u8>>::Output {
    &value
}
