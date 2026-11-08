#![feature(no_core)]
#![no_core]
extern crate base;
fn count<S: base::Searcher>(s: &S) -> u8 { s.haystack() }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { count(&base::Fault); 42 }
