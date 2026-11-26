#![feature(no_core)]
#![no_core]
extern crate base;
pub fn escape<'a>(_:&'a u32)->&'a u32 {
    let local=99;
    let outer=base::Outer(base::Inner{value:&local});
    base::borrowed(&outer)
}
