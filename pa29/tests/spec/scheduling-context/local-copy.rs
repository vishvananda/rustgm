#![feature(no_core)]
#![no_core]
extern crate base;
pub struct Local(usize);
impl base::Clone for Local { fn clone(&self) -> Self { *self } }
impl base::Copy for Local {}
const L: Local = base::id(Local(4));
fn take(_: &[u8; L.0]) -> i32 { 42 }
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { take(&[1, 2, 3, 4]) }
