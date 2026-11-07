#![feature(no_core)]
#![no_core]
extern crate base;
fn escape<'a>() -> base::Ref<'a, u8> { let v = 1u8; base::new_ref(&v) }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match *base::get(escape()) { 1 => 42, _ => 7 } }
