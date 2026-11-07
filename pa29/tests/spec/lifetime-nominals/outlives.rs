#![feature(no_core)]
#![no_core]
extern crate base;
fn bad<'a, T>(_x: &'a u8) -> u8 { let _w: base::Opt<base::Ref<'a, T>> = base::Opt::None; 42 }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let v = 1u8; match bad::<u8>(&v) { 42 => 42, _ => 7 } }
