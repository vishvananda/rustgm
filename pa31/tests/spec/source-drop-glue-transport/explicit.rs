#![feature(no_core)]
#![no_core]
extern crate base;

pub fn forbidden<T: base::Drop>(value: &mut T) {
    base::Drop::drop(value);
}
