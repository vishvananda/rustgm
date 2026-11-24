#![feature(no_core)]
#![no_core]
extern crate base;

const DIRECT: u8 = base::Frame::<u8>::MIN.0;
const CALL: u8 = base::byte_min();
const WORD_MIN: u16 = base::Frame::<u16>::MIN.0;
const BYTE_BITS: u8 = base::Frame::<u8>::BITS;
const WORD_BITS: u8 = base::Frame::<u16>::BITS;

pub fn direct() -> u8 { DIRECT }
pub fn call() -> u8 { CALL }
pub fn word_min() -> u16 { WORD_MIN }
pub fn byte_bits() -> u8 { BYTE_BITS }
pub fn word_bits() -> u8 { WORD_BITS }
