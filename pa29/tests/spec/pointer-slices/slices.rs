#![feature(no_core)]
#![no_core]
extern crate base;
fn relay(s: &[u8]) -> u8 { base::width(base::pass(s)) }
fn held(s: &[u8]) -> u8 { base::view(s).n }
fn generic(s: &[u8]) -> u8 { base::width(base::generic_slice(s)) }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { 42 }
