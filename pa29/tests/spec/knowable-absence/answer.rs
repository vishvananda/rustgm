#![feature(no_core)]
#![no_core]
extern crate base;
fn need<T: base::Error + ?base::Sized>() {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<&base::Fault>(); need::<&&base::Fault>(); 42 }
