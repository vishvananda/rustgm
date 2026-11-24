#![feature(no_core)]
#![no_core]
extern crate base;

fn reject(text: *const i8) -> usize {
    base::runtime::text_length(text)
}
