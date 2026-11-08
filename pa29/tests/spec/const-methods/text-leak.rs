#![feature(no_core)]
#![no_core]
extern crate base;
fn leak<'x>(h: &base::Held<'x>) -> &'static str { h.text() }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match base::Held::new("hi", 42).count() { _ => 7 } }
