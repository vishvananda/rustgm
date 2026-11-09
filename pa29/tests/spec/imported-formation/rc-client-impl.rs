#![feature(no_core)]
#![no_core]
extern crate base;
struct Mine;
impl base::Allocator for Mine {}
fn hold(_r: base::Rc<u8, Mine>) {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
