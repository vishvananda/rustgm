#![feature(no_core)]
#![no_core]
extern crate base;
fn fused<T: base::TrustedFused>() {}
fn count<S: base::Searcher>(s: &S) -> u8 { s.haystack() }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { fused::<base::Counter>(); fused::<base::Fuse<base::Counter>>(); let c = base::Counter { n: 42 }; match (count(&42u8), base::searched(&42u8), count(&c), base::searched(&c), <u8 as base::Searcher>::make(), <base::Counter as base::Searcher>::make()) { (42, 42, 42, 42, 42, 42) => 42, _ => 7 } }
