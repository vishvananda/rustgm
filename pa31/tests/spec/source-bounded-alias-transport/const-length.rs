#![feature(no_core)]
#![no_core]
extern crate base;

fn wrong_length(value: base::ExportedArray<u32, 3>) -> base::ExportedArray<u32, 4> {
    base::preserve_three(value)
}
