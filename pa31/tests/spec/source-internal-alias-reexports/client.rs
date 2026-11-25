#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed_base() -> base::PublicWord {
    base::read_base()
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_first() -> base::PublicWord {
    base::pair_first((23, 31))
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_second() -> u32 {
    base::pair_second((23, 31))
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_static() -> u32 {
    let value: base::PublicWord = base::BASE_WORD;
    match value {
        19 => 41,
        _ => 0,
    }
}
