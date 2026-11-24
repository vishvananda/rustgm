#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

struct Borrowed<'a>(&'a u8);

impl<'a> base::Borrowable<'a> for Borrowed<'a> {
    fn payload(&self) -> u8 { *self.0 }
}
fn forward<'a, T: base::Borrowable<'a>>(value: &T) -> u8 {
    value.payload()
}
#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u8 { forward(&Borrowed(&41)) }
