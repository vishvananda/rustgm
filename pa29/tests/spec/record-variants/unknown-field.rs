#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let s = base::Shape::Rect { w: 1, h: 2, d: 3 }; 0 }
