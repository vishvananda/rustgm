#![feature(no_core)]
#![no_core]
extern crate base;
static L: base::Loc<'static> = base::Loc { file: "here", line: 42 };
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match L.line { 42 => 42, _ => 7 } }
