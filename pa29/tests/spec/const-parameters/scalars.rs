#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let _a: base::Signed<-1> = base::minus_one(); let _b: base::Flag<true> = base::on(); let _c: base::Letter<'z'> = base::letter(); 42 }
