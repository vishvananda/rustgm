#![feature(no_core)]
#![no_core]
extern crate base;
fn widen<'a>(x: base::Contra<&'a u8>, _y: &'a u8) -> base::Contra<&'static u8> { x }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let v = 1u8; let _w = widen(base::Contra(base::PhantomData), &v); 42 }
