#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let a = base::pass([base::Opt::Some(42u8)]); let _b = a; let _c = a; 42 }
