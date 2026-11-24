#![feature(no_core)]
#![no_core]

extern crate base;

struct Local;

impl base::Value for Local {
    type Item<'a> = &'a u8
    where
        Self: 'a;
}

fn project<'a>(value: &'a u8) -> <Local as base::Value>::Item<'a> {
    value
}

#[unsafe(no_mangle)]
pub extern "C" fn observed(value: &u8) -> &u8 {
    project(value)
}

#[cfg(missing_family_bound)]
struct Borrowed<'b>(&'b u8);

#[cfg(missing_family_bound)]
impl<'b> base::Value for Borrowed<'b> {
    type Item<'a> = &'a u8;
}
