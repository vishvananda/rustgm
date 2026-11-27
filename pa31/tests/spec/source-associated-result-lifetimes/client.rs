#![feature(no_core)]
#![no_core]
extern crate base;

fn shared<'a>(value: &'a u8) -> <&'a u16 as base::Rebind<u8>>::Output {
    value
}

fn unique<'a>(value: &'a mut u8) -> <&'a mut u16 as base::Rebind<u8>>::Output {
    value
}

#[unsafe(no_mangle)]
pub extern "C" fn observed(value: u8) -> u8 {
    let mut result = 0;
    *unique(&mut result) = *shared(&value);
    result
}
