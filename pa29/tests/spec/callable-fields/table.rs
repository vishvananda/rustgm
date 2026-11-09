#![feature(no_core)]
#![no_core]
extern crate base;
unsafe fn call(_p: *const ()) -> i32 { 42 }
unsafe fn drop_it(_p: *const ()) {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let v = (); let p: *const () = &v; let t = base::Table { call, drop: drop_it }; unsafe { base::run(&t, p) } }
