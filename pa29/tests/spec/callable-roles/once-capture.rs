#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 {
    let hit = 42u8;
    let seen = base::apply(move |x| { let _kept = hit; x }, 41u8);
    match seen { 41 => base::apply(move |_x| hit, 0u8) as i32, _ => 7 }
}
