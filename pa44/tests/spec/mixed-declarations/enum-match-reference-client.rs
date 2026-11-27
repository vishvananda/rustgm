#![no_std]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match base::or(base::some(40u8), 1u8) { 40 => match base::or(base::none(), 2u8) { 2 => 42, _ => 8 }, _ => 7 }
}
