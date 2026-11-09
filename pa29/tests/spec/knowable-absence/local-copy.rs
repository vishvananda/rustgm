#![feature(no_core)]
#![no_core]
extern crate base;
struct Local;
trait Mine {}
impl<T: base::Copy> Mine for T {}
impl Mine for Local {}
fn need<T: Mine>() {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<Local>(); need::<u8>(); 42 }
