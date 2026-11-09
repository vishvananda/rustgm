#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let p = base::Pair { f: base::hold, v: 1 }; let q = p; let r = <base::Pair as base::Clone>::clone(&p); let _x = (q.f)(q.v); (r.f)(r.v) }
