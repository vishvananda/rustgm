#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let _m: base::PhantomData<u16> = base::marker(); 42 }
