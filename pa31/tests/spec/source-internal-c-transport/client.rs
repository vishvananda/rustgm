#![feature(no_core)]
#![no_core]
extern crate base;

const _: [u8; 7] = [0; base::read_pair::<0>(7, 0) as usize];

#[unsafe(no_mangle)]
pub extern "C" fn observed(left: u8, right: u8) -> u8 {
    base::read_pair::<0>(left, right)
}
