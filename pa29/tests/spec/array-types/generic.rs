#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let a = base::pass([42u8, 0, 0, 0]); let b = base::pass::<bool, 2>([true, false]); let p = &raw const a as *const u8; let q = &raw const b as *const bool; match unsafe { *p } { 42 => match unsafe { *q } { true => 42, false => 7 }, _ => 7 } }
