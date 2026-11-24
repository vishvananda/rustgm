#![feature(no_core, const_trait_impl)]
#![no_core]
extern crate base;

struct Values;

const impl base::Iterator for Values {
    type Item = u8;
    fn next(&mut self) -> base::Maybe<u8> { base::Maybe::Absent }
}

const fn measure() -> usize {
    let values = Values;
    let (lower, upper) = base::Iterator::size_hint(&values);
    match upper {
        base::Maybe::Absent => lower,
        base::Maybe::Present(_) => 99,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_runtime() -> u8 { measure() as u8 }

const HINT: usize = measure();

#[unsafe(no_mangle)]
pub extern "C" fn observed_const() -> u8 { HINT as u8 }
