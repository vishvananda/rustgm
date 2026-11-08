#![feature(no_core,negative_impls)]
#![no_core]
extern crate base;
fn need<T: base::Send>() {}
struct Mine;
impl !base::Send for Mine {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<Mine>(); 42 }
