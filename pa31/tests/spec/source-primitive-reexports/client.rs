#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed_bool() -> base::primitive::u32 {
    let flag: base::primitive::bool = true;
    let _: &base::primitive::str = "ok";
    if flag { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_u32() -> base::primitive::u32 {
    let value: base::primitive::u32 = 41;
    value
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_char() -> base::primitive::u32 {
    let letter: base::primitive::char = 'A';
    letter as base::primitive::u32
}

mod shadowed {
    pub struct bool;

    pub fn unshadowed_flag() -> ::base::primitive::bool {
        let _: bool = bool;
        true
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_max() -> base::primitive::u32 {
    if shadowed::unshadowed_flag() { 7 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_usize() -> base::primitive::usize {
    let count: base::primitive::usize = 9;
    count
}
