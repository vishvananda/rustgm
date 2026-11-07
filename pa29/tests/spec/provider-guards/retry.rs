#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match base::pick(&base::Opt::Some(1u8)) {
        1 => match base::pick(&base::Opt::Some(2u8)) {
            2 => match base::pick(&base::Opt::<u8>::None) {
                3 => match base::pick(&base::Opt::Some(42u8)) { 42 => 42, _ => 10 },
                _ => 9,
            },
            _ => 8,
        },
        _ => 7,
    }
}
