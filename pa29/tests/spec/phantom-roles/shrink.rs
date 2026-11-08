#![feature(no_core)]
#![no_core]
extern crate base;
fn shrink<'a, 'b: 'a>(x: base::PhantomData<&'b u8>, _y: &'a u8) -> base::PhantomData<&'a u8> { x }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let v = 1u8; let _m = shrink(base::PhantomData, &v); 42 }
