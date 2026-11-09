#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let a = [42u8, 0, 0, 0]; let c = a; let b = base::copied(&a); let d = base::Clone::clone(&b); let mut e = [0u8, 0, 0, 0]; base::store(&mut e, d); let p = &raw const e as *const u8; let q = &raw const c as *const u8; match unsafe { *p } { 42 => match unsafe { *q } { 42 => 42, _ => 7 }, _ => 7 } }
