#![feature(no_core)]
#![no_core]
extern crate base;
struct Local;
trait Mine {}
impl<T: base::Add<Local>> Mine for T {}
impl Mine for u8 {}
fn need<T: Mine>() {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { need::<u8>(); 42 }
