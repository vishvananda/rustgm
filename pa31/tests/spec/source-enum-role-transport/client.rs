#![feature(no_core)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed_option(input: bool) -> u8 {
    let value = if input { base::Maybe::Filled(41u8) } else { base::Maybe::Empty };
    match value { base::Maybe::Filled(value) => value, base::Maybe::Empty => 19 }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_result(input: bool) -> u8 {
    let value = if input { base::Outcome::Value(23u8) } else { base::Outcome::Error(37u8) };
    match value { base::Outcome::Value(value) => value, base::Outcome::Error(error) => error }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_ordering() -> i8 {
    match base::Compare::Less {
        base::Compare::Less => 73,
        base::Compare::Equal => 17,
        base::Compare::Greater => 29,
    }
}
