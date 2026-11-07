#![feature(no_core)]
#![no_core]
extern crate base;
fn escape<'x>(h: &base::Held<'x>) -> &'static str { h.text() }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let _ = escape(&base::Held::new("e", 1)); 42 }
