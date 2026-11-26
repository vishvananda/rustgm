#![feature(no_core)]
#![no_core]
extern crate base;
pub fn escape<'a>(_: &'a u32) -> &'a u32 {
    let local = base::Cell(99);
    base::borrowed(&local)
}
