#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed(mut value: u8) -> u8 {
    let shared = base::observe(&&value, 61);
    let unique = base::observe(&(&mut value), shared);
    base::observe(&base::Marked(0u8), unique)
}
