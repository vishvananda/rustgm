#![feature(no_core)]
#![no_core]
extern crate base;
struct Local;
impl base::Error for Local {}
fn need<T: base::Error + ?base::Sized>() {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<&base::Fault>(); need::<Local>(); need::<&Local>(); 42 }
