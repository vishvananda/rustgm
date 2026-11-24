#![feature(no_core)]
#![no_core]
extern crate base;

pub fn observed() -> u32 {
    let value: base::ExportedArray<u32, 3> = base::ArrayStorage { values: [8, 13, 34] };
    let _: base::ExportedArray<u32, 3> = base::preserve_three(value);
    34
}

fn relay<const M: usize>(value: base::ExportedArray<u32, M>) -> base::ExportedArray<u32, M> {
    value
}
