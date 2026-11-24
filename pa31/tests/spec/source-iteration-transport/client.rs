#![feature(no_core)]
#![no_core]
extern crate base;

struct Values { pending: bool, value: u8 }

impl base::Iterator for Values {
    type Item = u8;
    fn next(&mut self) -> base::Maybe<u8> {
        if self.pending {
            self.pending = false;
            base::Maybe::Present(self.value)
        } else {
            base::Maybe::Absent
        }
    }
}

impl base::IntoIterator for Values {
    type Item = u8;
    type IntoIter = Self;
    fn into_iter(self) -> Self { self }
}

fn consume(pending: bool, value: u8) -> u8 {
    let mut last = 41u8;
    for item in (Values { pending, value }) {
        last = item;
    }
    last
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_value() -> u8 { consume(true, 7u8) }

#[unsafe(no_mangle)]
pub extern "C" fn observed_empty() -> u8 { consume(false, 7u8) }
