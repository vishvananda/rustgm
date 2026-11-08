#![feature(no_core,negative_impls)]
#![no_core]
extern crate base;
struct Mine;
unsafe impl !base::Searcher for Mine {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
