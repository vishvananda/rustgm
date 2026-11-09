#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let () = base::NOTHING;
    match base::BIG { 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF => 42, _ => 1 }
}
