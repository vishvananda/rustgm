#![feature(no_core)]
#![no_core]
extern crate base;
trait LocalAlloc {}
struct LocalRc<A: LocalAlloc> { a: A }
fn hold<A>(_r: LocalRc<A>) {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
