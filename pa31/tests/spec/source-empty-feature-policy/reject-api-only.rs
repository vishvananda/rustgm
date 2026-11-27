#![feature(no_core, predicate_api)]
#![no_core]

pub fn rejected(value: u8) -> u8 {
    base::gated(value)
}
