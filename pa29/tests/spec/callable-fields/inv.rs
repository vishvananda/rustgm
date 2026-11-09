#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let _i: base::Inv<u8> = base::Inv(base::PhantomData); let _s: base::Inv<[u8]> = base::Inv(base::PhantomData); 42 }
