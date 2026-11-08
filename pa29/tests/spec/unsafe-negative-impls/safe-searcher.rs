#![feature(no_core)]
#![no_core]
extern crate base;
struct Mine;
impl base::Searcher for Mine { fn haystack(&self) -> u8 { 42 } fn make() -> u8 { 42 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
