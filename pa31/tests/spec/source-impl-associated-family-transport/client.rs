#![feature(no_core)]
#![no_core]
extern crate base;

fn project<'a, 'b>(
    left: &'a u8,
    right: &'b u8,
) -> <base::Needle<'b> as base::Pattern>::Searcher<'a> {
    base::Pair(left, right)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed<'a, 'b>(left: &'a u8, right: &'b u8) -> &'a u8 {
    project(left, right).0
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_captured<'a, 'b>(left: &'a u8, right: &'b u8) -> &'b u8 {
    project(left, right).1
}

#[cfg(wrong_capture)]
pub fn wrong_capture<'a, 'b>(left: &'a u8, right: &'b u8) -> &'b u8 {
    project(left, right).0
}
