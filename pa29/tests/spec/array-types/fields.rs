#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let b = base::bytes(base::Buf { bytes: [42, 0, 0, 0], n: 1 }); let g = base::rows(base::Grid([[42, 0], [0, 0]])); let p = &raw const b as *const u8; let q = &raw const g as *const u8; match unsafe { *p } { 42 => match unsafe { *q } { 42 => 42, _ => 7 }, _ => 7 } }
