#![feature(no_core)]
#![no_core]
extern crate base;
fn fused<T: base::TrustedFused>() {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { fused::<base::Fuse<u8>>(); 42 }
