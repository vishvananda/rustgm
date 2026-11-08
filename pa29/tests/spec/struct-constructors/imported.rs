#![feature(no_core)]
#![no_core]
extern crate base;
use base::{Unit, Wrap as W};
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let f = base::Wrap; match base::unwrap(f(W(42).0)) { 42 => match base::accept(Unit) { 42 => 42, _ => 8 }, _ => 7 } }
