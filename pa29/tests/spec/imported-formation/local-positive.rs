#![feature(no_core)]
#![no_core]
extern crate base;
trait LocalAlloc {}
struct Yes;
impl LocalAlloc for Yes {}
struct LocalRc<A: LocalAlloc> { a: A }
fn hold(_r: LocalRc<Yes>) {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
