#![feature(no_core)]
#![no_core]
extern crate base;
fn checked(){}
#[expect(deprecated)]fn untouched()->u8{base::gated::marked()}
