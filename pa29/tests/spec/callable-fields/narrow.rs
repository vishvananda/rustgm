#![feature(no_core)]
#![no_core]
extern crate base;
fn narrow<'a>(x: base::Contra<&'static u8>, _y: &'a u8) -> base::Contra<&'a u8> { x }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let v = 1u8; let _n = narrow(base::Contra(base::PhantomData), &v); 42 }
