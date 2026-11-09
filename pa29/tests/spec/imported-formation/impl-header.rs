#![feature(no_core)]
#![no_core]
extern crate base;
struct NotAlloc;
trait Mine {}
impl Mine for base::Rc<u8, NotAlloc> {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
