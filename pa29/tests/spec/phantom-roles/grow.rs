#![feature(no_core)]
#![no_core]
extern crate base;
fn grow<'a, 'b: 'a>(x: base::PhantomData<&'a u8>) -> base::PhantomData<&'b u8> { x }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let _m = grow(base::PhantomData); 42 }
