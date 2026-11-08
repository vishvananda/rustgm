#![feature(no_core)]
#![no_core]
extern crate base;
fn fused<T: base::TrustedFused>() {}
struct Mine;
unsafe impl base::TrustedFused for Mine {}
unsafe impl base::Searcher for Mine { fn haystack(&self) -> u8 { 42 } fn make() -> u8 { 42 } }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { fused::<base::Fuse<Mine>>(); match base::searched(&Mine) { 42 => 42, _ => 7 } }
