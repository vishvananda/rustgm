#![feature(no_core)]
#![no_core]
extern crate base;
trait LocalAlloc {}
struct LocalRc<A: LocalAlloc> { a: A }
struct NotAlloc;
struct Holder { r: LocalRc<NotAlloc> }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
