#![feature(no_core, asm_native_fixture)]
#![no_core]
extern crate base;

pub fn rejected() {
    base::arch::asm!("nop");
}
