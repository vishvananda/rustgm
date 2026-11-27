#![no_std]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    match (base::LANES, base::FLAG, base::LETTER, base::NEG as u8) { (4, true, 'x', 255) => 42, _ => 1 }
}
